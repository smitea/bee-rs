package com.enmotech.nirvana.bee.connector.codec;

import io.netty.buffer.ByteBuf;

import java.io.ByteArrayInputStream;

public interface Decoder extends Protocol {
    void decode(ByteBuf packet) throws Exception;

    default String read256String(ByteBuf packet) throws Exception {
        int len = packet.readByte();
        return readString(packet, len);
    }

    default String readLongString(ByteBuf packet) throws Exception {
        long len = packet.readUnsignedInt();
        return readString(packet, (int) len);
    }

    default Boolean readBoolean(ByteBuf packet) throws Exception {
        return packet.readByte() == 1;
    }

    default Long readInteger(ByteBuf packet) throws Exception {
        return packet.readLong();
    }

    default Double readDouble(ByteBuf packet) throws Exception {
        return packet.readDouble();
    }

    default Bytes readBytes(ByteBuf packet) throws Exception {
        long len = packet.readUnsignedInt();
        byte[] bytes = new byte[(int) len];
        packet.readBytes(bytes);
        return new Bytes(len, new ByteArrayInputStream(bytes));
    }

    default String readString(ByteBuf packet, int len) throws Exception {
        byte[] bytes = new byte[len];
        packet.readBytes(bytes);
        return new String(bytes);
    }

    default void assertCode(ByteBuf buf) throws BeeException {
        int code = buf.readInt();
        try {
            String msg = read256String(buf);
            if (code != 0x00) {
                throw new BeeException(code, msg);
            }
        } catch (BeeException e) {
            throw e;
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg, e);
        }
    }
}
