package com.enmotech.nirvana.bee.connector.codec;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;

public class ConnectReq implements Encoder {
    private final String url;
    private final String application;
    private final ConnectResp resp = new ConnectResp();

    public ConnectReq(String url, String application) {
        this.url = url;
        this.application = application;
    }

    @Override
    public ByteBuf encode() throws Exception {
        ByteBuf buf = Unpooled.directBuffer();
        writeString(buf, url);
        writeString(buf, application);
        return buf;
    }

    @Override
    public boolean valid(ByteBuf byteBuf) {
        try {
            resp.decode(byteBuf);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    @Override
    public int type() {
        return 0x00;
    }

    public String getUrl() {
        return url;
    }

    public String getApplication() {
        return application;
    }
}
