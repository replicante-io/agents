####################
# Build the agents #
####################
ARG RUST_VERSION=1.40.0
FROM rust:$RUST_VERSION as builder

# Add the code.
COPY . /code

# Compile agents.
RUN cargo build --manifest-path /code/agents/mongodb/Cargo.toml --release --locked
RUN cargo build --manifest-path /code/agents/zookeeper/Cargo.toml --release --locked


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

# Copy binaries from builder to smaller image.
COPY --from=builder /code/agents/mongodb/target/release/replicante-agent-mongodb /opt/replicante/bin/
COPY --from=builder /code/agents/zookeeper/target/release/replicante-agent-zookeeper /opt/replicante/bin/

# Set up runtime environment as needed.
ENV PATH=/opt/replicante/bin:$PATH
USER $REPLI_UNAME
WORKDIR /home/replicante

# Validate binaries.
RUN /opt/replicante/bin/replicante-agent-mongodb --version \
    && /opt/replicante/bin/replicante-agent-zookeeper --version
