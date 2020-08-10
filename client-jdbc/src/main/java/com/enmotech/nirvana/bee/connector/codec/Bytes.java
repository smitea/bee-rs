package com.enmotech.nirvana.bee.connector.codec;

import java.io.Closeable;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.sql.Blob;
import java.sql.SQLException;

public class Bytes implements Closeable, Blob {
    private final long size;
    private final InputStream stream;

    public Bytes(long size, InputStream stream) {
        this.size = size;
        this.stream = stream;
    }

    @Override
    public void close() throws IOException {
        stream.close();
    }

    @Override
    public long length() throws SQLException {
        return size;
    }

    @Override
    public byte[] getBytes(long pos, int length) throws SQLException {
        byte[] bytes = new byte[length];
        try {
            int size = this.stream.read(bytes, (int) pos, length);
        } catch (IOException e) {
            throw new BeeException(e);
        }
        return bytes;
    }

    @Override
    public InputStream getBinaryStream() throws SQLException {
        return stream;
    }

    @Override
    public long position(byte[] pattern, long start) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public long position(Blob pattern, long start) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int setBytes(long pos, byte[] bytes) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public int setBytes(long pos, byte[] bytes, int offset, int len) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public OutputStream setBinaryStream(long pos) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void truncate(long len) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void free() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public InputStream getBinaryStream(long pos, long length) throws SQLException {
        throw new NotSupportException();
    }

    public byte[] toBytes() throws IOException {
        byte[] bytes = new byte[(int) size];
        int len = stream.read(bytes);
        return bytes;
    }
}
