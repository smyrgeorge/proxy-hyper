FROM alpine:3.13
LABEL maintainer="smyrgeroge@gmail.com"

RUN apk add --no-cache libgcc ca-certificates openssl openssl-dev && \
    update-ca-certificates

ADD config /opt/payload/config
ADD target/release/proxy-hyper /opt/payload/proxy-hyper
WORKDIR /opt/payload

EXPOSE 80

CMD ["./proxy-hyper"]
