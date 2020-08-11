package com.enmotech.nirvana.bee.connector.codec;

import com.enmotech.nirvana.bee.connector.DataType;
import io.netty.buffer.ByteBuf;

import java.io.ByteArrayInputStream;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.Delayed;

public interface Encoder extends Protocol {
    ByteBuf encode() throws Exception;

    boolean valid(ByteBuf byteBuf);

    default boolean isMulti() {
        return false;
    }

    default void writeString(ByteBuf packet, String value) throws Exception {
        packet.writeByte(DataType.STRING.getType());
        packet.writeInt(value.length());
        packet.writeBytes(value.getBytes(StandardCharsets.UTF_8));
    }

    default void writeInteger(ByteBuf packet, long value) throws Exception {
        writeInteger(packet, Long.valueOf(value));
    }

    default void writeInteger(ByteBuf packet, int value) throws Exception {
        writeInteger(packet, Long.valueOf(value));
    }

    default void writeInteger(ByteBuf packet, Long value) throws Exception {
        packet.writeByte(DataType.INTEGER.getType());
        packet.writeLong(value);
    }

    default void writeDouble(ByteBuf packet, Double value) throws Exception {
        packet.writeByte(DataType.NUMBER.getType());
        packet.writeDouble(value);
    }
}
