package com.enmotech.nirvana.bee.connector;

import io.netty.buffer.ByteBuf;

class ResponsePacketHandler implements PacketHandler {
    private final StatementResp resp;
    private final StatementReq req;

    public ResponsePacketHandler(StatementResp resp, StatementReq req) {
        this.resp = resp;
        this.req = req;
    }

    @Override
    public void decode(ByteBuf packet) {
        resp.decode(packet);
    }

    @Override
    public boolean validPacket(ByteBuf packet) {
        try {
            StatementResp resp = new StatementResp();
            resp.decode(packet);
            return req.getId() == resp.getId();
        } catch (Exception e) {
            return false;
        }
    }

    @Override
    public int type() {
        return resp.type();
    }

    @Override
    public boolean isMulti() {
        return true;
    }

    @Override
    public boolean isEnd() {
        try {
            return resp.isAbort();
        } catch (Exception e) {
            return false;
        }
    }
}