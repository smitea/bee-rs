package com.enmotech.nirvana.bee.connector.codec;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;

public class StatementReq implements Encoder {
    private final int id;
    private final String script;
    private final int timeout;

    public StatementReq(int id, String script, int timeout) {
        this.id = id;
        this.script = script;
        this.timeout = timeout;
    }

    @Override
    public ByteBuf encode() throws Exception {
        ByteBuf buf = Unpooled.buffer();
        writeInteger(buf, id);
        writeString(buf, script);
        writeInteger(buf, timeout);
        return buf;
    }

    @Override
    public boolean valid(ByteBuf byteBuf) {
        try {
            StatementResp resp = new StatementResp();
            resp.decode(byteBuf);
            return id == resp.getId();
        } catch (Exception e) {
            return false;
        }
    }

    @Override
    public int type() {
        return 0x02;
    }

    public String getScript() {
        return script;
    }

    public int getId() {
        return id;
    }

    public int getTimeout() {
        return timeout;
    }
}
