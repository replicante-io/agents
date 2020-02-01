####################
# Build the agents #
####################
ARG RUST_VERSION=1.39.0
FROM rust:$RUST_VERSION as builder

# Add the code.
COPY . /code

# Compile agents.
RUN cd /code/mongodb && cargo build --release --locked \
    && cd /code/zookeeper && cargo build --release --locked


#######################################
# Package agents into a smaller image #
#######################################
FROM debian:buster-slim

# Create a replicante user to avoid using root.
ARG REPLI_GID=1616
ARG REPLI_GNAME=replicante
ARG REPLI_UID=1616
ARG REPLI_UNAME=replicante
RUN addgroup --gid $REPLI_GID $REPLI_GNAME \
    && adduser --disabled-login --disabled-password --system --uid $REPLI_UID --gid $REPLI_GID $REPLI_UNAME

# Install needed runtime dependencies.
RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y libssl1.1 \
    && apt-get clean all

# Install tini supervisor
ARG TINI_VERSION=v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini
ENTRYPOINT ["/tini", "--"]

# Copy binaries from builder to smaller image.
COPY --from=builder /code/target/release/replicante-agent-mongodb /opt/replicante/bin/replicante-agent-mongodb
COPY --from=builder /code/target/release/replicante-agent-zookeeper /opt/replicante/bin/replicante-agent-zookeeper

# Set up runtime environment as needed.
ENV PATH=/opt/replicante/bin:$PATH
USER $REPLI_UNAME

# Validate binaries.
RUN /opt/replicante/bin/replicante-agent-mongodb --version \
    && /opt/replicante/bin/replicante-agent-zookeeper --version
