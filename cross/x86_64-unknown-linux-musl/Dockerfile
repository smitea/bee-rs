FROM rustembedded/cross:x86_64-unknown-linux-musl-0.2.1

RUN apt-get update && apt-get install -y libssl-dev
ENV OPENSSL_STATIC=true
ENV OPENSSL_LIB_DIR=/usr/local/lib
ENV OPENSSL_INCLUDE_DIR=/usr/local/include
ENV SODIUM_STATIC=true
ENV SODIUM_LIB_DIR=/usr/local/lib
ENV SODIUM_INCLUDE_DIR=/usr/local/include