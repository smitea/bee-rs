package com.enmotech.nirvana.bee.connector;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.ByteToMessageDecoder;

import java.util.Arrays;
import java.util.List;

class PacketDecoder extends ByteToMessageDecoder {

    @Override
    protected void decode(ChannelHandlerContext channelHandlerContext, ByteBuf in, List<Object> list) throws Exception {
        in.markReaderIndex();
        if (in.readableBytes() < Packet.LENGTH) {
            return;
        }

        byte[] head = new byte[Packet.HEAD.length];
        in.readBytes(head);

        if (!Arrays.equals(head, Packet.HEAD)) {
            in.resetReaderIndex();
            return;
        }

        byte type = in.readByte();
        long len = in.readLong();

        if (in.readerIndex() > in.writerIndex() - (len + 10)) {
            in.resetReaderIndex();
            return;
        }
        ByteBuf data = Unpooled.buffer((int) len);
        in.readBytes(data);
        long crc = in.readLong();
        if (!(crc == len + Packet.LENGTH)) {
            in.resetReaderIndex();
            return;
        }

        byte[] end = new byte[2];
        in.readBytes(end);
        if (!(Arrays.equals(end, Packet.END))) {
            in.resetReaderIndex();
            return;
        }
        Packet packet = new Packet(type, data);
        list.add(packet);
    }
}
