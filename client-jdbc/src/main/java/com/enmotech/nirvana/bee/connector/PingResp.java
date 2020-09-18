package com.enmotech.nirvana.bee.connector;

import io.netty.buffer.ByteBuf;

class PingResp implements Decoder{

    @Override
    public int type() {
        return 0x07;
    }

    @Override
    public void decode(ByteBuf packet) throws Exception {
        
    }
}
