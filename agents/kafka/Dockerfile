###################
# Build the agent #
###################
ARG RUST_VERSION=1.40.0
FROM rust:$RUST_VERSION as builder

# Install Java and build kafka agent.
RUN DEBIAN_FRONTEND=noninteractive apt-get update
RUN apt-get install -y default-jdk

# Add and compile the code.
COPY . /code
RUN cargo build --manifest-path /code/agents/kafka/Cargo.toml --release --locked

# NOTE:
#   The j4rs runtime needs some of the files in deps but not all.
#   The needed files have dyanmically generated names which we cannot easily
#   predict for a directed COPY command in upper layers.
#   Create a copy of the deps directory that only includes the needed files
#   so that we can COPY that over to upper layers and reduce image size.
RUN cd /code/agents/kafka/target/release \
    && cp -r deps deps-slim \
    && cd deps-slim \
    && ls -1 . | grep -v j4rs | xargs -l rm


#######################################
# Package agents into a smaller image #
#######################################
FROM openjdk:8-jre-slim

# Create a replicante user to avoid using root.
ARG REPLI_GID=1616
ARG REPLI_GNAME=replicante
ARG REPLI_UID=1616
ARG REPLI_UNAME=replicante
RUN addgroup --gid $REPLI_GID $REPLI_GNAME \
    && adduser --disabled-login --disabled-password --system --uid $REPLI_UID --gid $REPLI_GID $REPLI_UNAME

# Copy binaries from builder to smaller image.
COPY --from=builder /code/agents/kafka/target/release/deps-slim /opt/replicante/bin/deps
COPY --from=builder /code/agents/kafka/target/release/jassets /opt/replicante/bin/jassets
COPY --from=builder /code/agents/kafka/target/release/replicante-agent-kafka /opt/replicante/bin/

# Set up runtime environment as needed.
#  Add libjvm for the kafka agent.
ENV LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/local/openjdk-8/lib/amd64/server
ENV PATH=/opt/replicante/bin:$PATH
USER $REPLI_UNAME
WORKDIR /home/replicante
CMD ["/opt/replicante/bin/replicante-agent-kafka"]

# Validate binaries
RUN /opt/replicante/bin/replicante-agent-kafka --version
