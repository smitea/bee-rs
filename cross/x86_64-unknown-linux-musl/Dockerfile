FROM rustembedded/cross:x86_64-unknown-linux-musl-0.1.16

RUN apt-get update && \
    apt-get install -y python curl
RUN curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py
RUN python --version
RUN python get-pip.py
RUN pip install abimap