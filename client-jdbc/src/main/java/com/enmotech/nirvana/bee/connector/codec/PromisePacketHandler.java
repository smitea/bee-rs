package com.enmotech.nirvana.bee.connector.codec;

import com.enmotech.nirvana.bee.connector.promise.Promise;
import io.netty.buffer.ByteBuf;

public class PromisePacketHandler<T extends Decoder, Encode extends Encoder> implements PacketHandler {
    private Promise<T> promise;
    private T decoder;
    private Encode encoder;

    public PromisePacketHandler(Promise<T> promise, T decoder, Encode encoder) {
        this.promise = promise;
        this.decoder = decoder;
        this.encoder = encoder;
    }

    @Override
    public void handle() {
        promise.onSuccess(decoder);
    }

    @Override
    public int type() {
        return decoder.type();
    }

    @Override
    public boolean isMulti() {
        return encoder.isMulti();
    }

    @Override
    public void decode(ByteBuf packet) {
        try {
            decoder.decode(packet);
        } catch (IndexOutOfBoundsException ignored) {
        } catch (Exception e) {
            promise.onFailure(e);
        }
    }

    @Override
    public boolean validPacket(ByteBuf packet) {
        return encoder.valid(packet);
    }
}
