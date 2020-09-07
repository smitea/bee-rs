package com.enmotech.nirvana.bee.connector;

import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToByteEncoder;

class PacketEncoder extends MessageToByteEncoder<Packet> {

    @Override
    protected void encode(ChannelHandlerContext channelHandlerContext, Packet packet, ByteBuf out) throws Exception {
        out.writeBytes(Packet.HEAD);
        out.writeByte(packet.getType());
        out.writeLong(packet.getLen());
        if (packet.getLen() != 0) {
            out.writeBytes(packet.getData());
        }
        out.writeLong(packet.getCrc());
        out.writeBytes(Packet.END);

        channelHandlerContext.flush();
    }
}
