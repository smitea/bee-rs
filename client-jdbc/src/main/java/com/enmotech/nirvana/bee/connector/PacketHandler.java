package com.enmotech.nirvana.bee.connector;

import io.netty.buffer.ByteBuf;

interface PacketHandler{
    void decode(ByteBuf packet);

    boolean validPacket(ByteBuf packet);

    int type();

    default boolean isMulti(){
        return false;
    }

    default boolean isEnd() {
        return true;
    }

    default void handle(){

    }
}
