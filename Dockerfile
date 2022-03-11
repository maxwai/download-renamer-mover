FROM gradle:7.4.1-jdk17-alpine

COPY --chown=gradle:gradle . /home/gradle/project
WORKDIR /home/gradle/project
ENTRYPOINT ["gradle", "run"]
#RUN gradle jar --no-daemon
#
#FROM openjdk:17-slim
#
#RUN mkdir /app
#
#COPY --from=build /home/gradle/project/build/libs/*.jar /app/download_watcher.jar
#
#ENTRYPOINT ["java", "-jar", "/app/download_watcher.jar", "$SERVER"]