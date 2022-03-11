FROM gradle:7.4.1-jdk17-alpine

COPY --chown=gradle:gradle . /home/gradle/project
WORKDIR /home/gradle/project
ENTRYPOINT gradle run --no-daemon