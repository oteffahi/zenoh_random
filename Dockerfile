FROM rust:1-slim-bullseye as build
WORKDIR /zenoh_random
COPY ./ .
RUN cargo build --release

FROM ubuntu:latest
COPY --from=build /zenoh_random/target/release/publisher /bin/publisher
COPY --from=build /zenoh_random/target/release/sub_callback /bin/sub_callback
COPY --from=build /zenoh_random/target/release/sub_stream /bin/sub_stream

RUN chmod +x /bin/publisher /bin/sub_callback /bin/sub_stream

EXPOSE 7447/tcp

CMD ["bash"]