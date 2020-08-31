package com.enmotech.nirvana.bee.connector.codec;

import com.enmotech.nirvana.bee.connector.ColumnInfo;
import com.enmotech.nirvana.bee.connector.DataType;
import com.enmotech.nirvana.bee.connector.Response;
import com.enmotech.nirvana.bee.connector.ResultRow;
import com.enmotech.nirvana.bee.connector.Value;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;

import java.nio.charset.Charset;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class StatementResp implements Decoder, Response {
    private Long id;

    private ColumnInfo[] header = null;
    private final AtomicBoolean hasHeader = new AtomicBoolean(false);
    private final AtomicBoolean hasAbort = new AtomicBoolean(false);
    private final BlockingQueue<ResultRow> rowQueue = new ArrayBlockingQueue<>(1024);

    private volatile BeeException exception = null;

    private final Lock lock = new ReentrantLock();
    private final Condition headSignal = lock.newCondition();
    private final Condition rowSignal = lock.newCondition();

    private final Long timeout;
    private final TimeUnit unit;

    public StatementResp() {
        this.timeout = 5L;
        this.unit = TimeUnit.SECONDS;
    }

    public StatementResp(Long timeout, TimeUnit unit) {
        this.timeout = timeout;
        this.unit = unit;
    }

    @Override
    public int type() {
        return 0x03;
    }

    @Override
    public void decode(ByteBuf packet) {
        try {
            packet.markReaderIndex();
            id = packet.readUnsignedInt();
            CallState state = CallState.valueOf(packet.readByte());
            switch (state) {
                case Columns:
                    ColumnInfo[] header = decodeHeader(packet);
                    try {
                        lock.lock();
                        this.header = header;
                        hasHeader.set(true);
                        headSignal.signalAll();
                    } finally {
                        lock.unlock();
                    }
                    break;
                case Row:
                    Value[] values = decodeRow(packet);
                    try {
                        lock.lock();
                        if (this.header != null && values.length == this.header.length) {
                            rowQueue.offer(new ResultRow(this.header, values));
                        }
                        rowSignal.signal();
                    } finally {
                        lock.unlock();
                    }
                    break;
                case Error:
                    try {
                        lock.lock();
                        validError(packet);
                    } finally {
                        lock.unlock();
                    }
                case Abort:
                    try {
                        lock.lock();
                        headSignal.signal();
                        rowSignal.signal();
                        hasAbort.set(true);
                    } finally {
                        lock.unlock();
                    }
                    break;
            }
        } catch (IndexOutOfBoundsException e) {
            packet.resetReaderIndex();
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            lock.lock();
            exception = new BeeException(msg,e);
            hasAbort.set(true);

            headSignal.signal();
            rowSignal.signal();

            lock.unlock();
        }
    }

    @Override
    public ColumnInfo[] getColumns() throws BeeException {
        try {
            lock.lock();
            if (!hasHeader.get()) {
                if (!headSignal.await(timeout, unit)) {
                    throw new TimeoutException("await timeout");
                }
            }
            if (header != null) {
                return this.header;
            } else {
                return getColumns();
            }
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg, e);
        } finally {
            lock.unlock();
            checkThrowable();
        }
    }

    @Override
    public ResultRow next() throws BeeException {
        try {
            lock.lock();
            ResultRow row = rowQueue.poll();
            if (row == null) {
                if (hasNext()) {
                    return rowQueue.poll();
                } else {
                    throw new BeeException(-1, "Result row is empty!");
                }
            }
            return row;
        } finally {
            lock.unlock();
            checkThrowable();
        }
    }

    @Override
    public boolean hasNext() throws BeeException {
        try {
            lock.lock();
            boolean abort = this.hasAbort.get();
            if (!abort && rowQueue.isEmpty()) {
                if (!rowSignal.await(timeout, unit)) {
                    throw new TimeoutException();
                }
            }
            abort = this.hasAbort.get();
            if (abort){
                return !rowQueue.isEmpty();
            }
            return true;
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg,e);
        } finally {
            lock.unlock();
            checkThrowable();
        }
    }

    private void checkThrowable() throws BeeException {
        if (exception != null) {
            throw exception;
        }
    }

    private ColumnInfo[] decodeHeader(ByteBuf data) {
        int headSize = data.readByte();
        ColumnInfo[] header = new ColumnInfo[headSize];
        for (int i = 0; i < headSize; i++) {
            int dataLen = data.readByte();
            ByteBuf bytes = data.readBytes(dataLen);
            String name = bytes.toString(Charset.defaultCharset());
            bytes.release();
            int type = data.readByte();
            header[i] = new ColumnInfo(name, DataType.valueOf(type));
        }
        return header;
    }

    private Value[] decodeRow(ByteBuf data) throws BeeException {
        int rowSize = data.readByte();
        Value[] values = new Value[rowSize];
        try {
            for (int i = 0; i < rowSize; i++) {
                int type = data.readByte();
                DataType dataType = DataType.valueOf(type);
                switch (dataType) {
                    case BOOLEAN:
                        values[i] = Value.bool(readBoolean(data));
                        break;
                    case NIL:
                        values[i] = Value.nil();
                        break;
                    case INTEGER:
                        values[i] = Value.integer(readInteger(data));
                        break;
                    case STRING:
                        values[i] = Value.str(readLongString(data));
                        break;
                    case NUMBER:
                        values[i] = Value.number(readDouble(data));
                        break;
                    case BYTES:
                        values[i] = Value.bytes(readBytes(data));
                        break;
                }
            }
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg,e);
        }
        return values;
    }

    private void validError(ByteBuf packet) throws BeeException {
        assertCode(packet);
    }

    public boolean isAbort() {
        return hasAbort.get();
    }

    public enum CallState {
        Columns(0x00),
        Row(0x01),
        Abort(0x02),
        Error(0x03);

        private final int code;

        CallState(int code) {
            this.code = code;
        }

        static CallState valueOf(int code) {
            for (CallState state : values()) {
                if (state.code == code) {
                    return state;
                }
            }
            return CallState.Abort;
        }
    }

    public Long getId() {
        return id;
    }
}
