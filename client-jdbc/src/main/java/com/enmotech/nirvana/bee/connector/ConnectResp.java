package com.enmotech.nirvana.bee.connector;

import java.util.concurrent.atomic.AtomicBoolean;

import io.netty.buffer.ByteBuf;

class ConnectResp implements Decoder {
    private final AtomicBoolean isOk = new AtomicBoolean(false);
    private BeeException exception;

    @Override
    public void decode(ByteBuf buf) throws Exception {
        try {
            int state = buf.readByte();
            if (state == 0x01) {
                assertCode(buf);
            } else {
                isOk.set(true);;
            }
        } catch (BeeException e) {
            isOk.set(false);;
            exception = e;
        }
    }

    @Override
    public int type() {
        return 0x01;
    }

    public BeeException getException(){
        return this.exception;
    }

    public boolean isOk(){
        return this.isOk.get();
    }
}
