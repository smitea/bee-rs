package com.enmotech.nirvana.bee.connector.codec;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;

public class Packet {
    static byte[] HEAD = {(byte) 0xFF, (byte) 0xFF};
    static byte[] END = {0x0D,0x0A};
    public static int LENGTH = 21;

    private final int type;
    private final long len;
    private final ByteBuf data;
    private final long crc;

    public Packet(int type, ByteBuf data) {
        ByteBuf buffer =  Unpooled.wrappedBuffer(data.nioBuffer());
        this.type = type;
        this.len = buffer.writerIndex();
        this.data = Unpooled.buffer(buffer.writerIndex()).writeBytes(buffer);
        this.crc = this.len + LENGTH;
    }

    public int getType() {
        return type;
    }

    public long getLen() {
        return len;
    }

    public ByteBuf getData() {
        return data;
    }

    public long getCrc() {
        return crc;
    }

    @Override
    public String toString() {
        return "Packet{" +
                "type=" + type +
                ", len=" + len +
                ", data=" + ByteBufUtil.hexDump(data) +
                ", crc=" + crc +
                '}';
    }
}
