FROM ubuntu

RUN apt update
RUN apt install -y build-essential
RUN apt install -y verilator
RUN apt install -y perl-doc
RUN apt install -y file
RUN apt install -y cmake
RUN apt install -y man-db
RUN sh -c 'yes | unminimize'
RUN apt install -y curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/install_rustup
RUN sh /tmp/install_rustup -y
