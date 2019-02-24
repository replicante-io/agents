###################
# Build the agent #
###################
ARG RUST_VERSION=1.32.0
FROM rust:$RUST_VERSION as builder

# Install Java and build kafka agent.
RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y default-jdk

# Add and compile the code.
COPY . /code
RUN export JAVA_HOME="/usr/lib/jvm/java-8-openjdk-amd64" \
    && export LD_LIBRARY_PATH="${JAVA_HOME}/jre/lib/amd64/server:$LD_LIBRARY_PATH" \
    && cd /code/kafka \
    && cargo build --release --locked


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

# Install tini supervisor
ARG TINI_VERSION=v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini
ENTRYPOINT ["/tini", "--"]

# Copy binaries from builder to smaller image.
COPY --from=builder /code/kafka/target/release/replicante-agent-kafka /opt/replicante/bin/replicante-agent-kafka
COPY --from=builder /code/kafka/target/release/deps /opt/replicante/bin/deps
COPY --from=builder /code/kafka/target/release/jassets/j4rs-0.1.5-jar-with-dependencies.jar /opt/replicante/bin/jassets/j4rs-0.1.5-jar-with-dependencies.jar

# Set up runtime environment as needed.
#  Add libjvm for the kafka agent.
ENV LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/lib/jvm/java-8-openjdk-amd64/jre/lib/amd64/server
ENV PATH=/opt/replicante/bin:$PATH
USER $REPLI_UNAME
CMD ["/opt/replicante/bin/replicante-agent-kafka"]

# Validate binaries
RUN /opt/replicante/bin/replicante-agent-kafka --version
