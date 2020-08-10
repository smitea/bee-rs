package com.enmotech.nirvana.bee.connector.codec;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;

public class ConnectResp implements Decoder {

    @Override
    public void decode(ByteBuf buf) throws Exception {
        int state = buf.readByte();
        if (state == 0x01) {
            assertCode(buf);
        }
    }

    @Override
    public int type() {
        return 0x01;
    }
}
