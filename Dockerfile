FROM gradle:7.4.1-jdk17-alpine

# download usermod and groupmod
RUN apk --no-cache add shadow runuser

# initialize user as needed
RUN adduser -Du 1001 -s /bin/sh abc

# set the work dir
WORKDIR /home/gradle/project

# Copy the gradle project
COPY --chown=gradle:gradle . /home/gradle/project

# Fix permissions
RUN chmod +x entrypoint.sh

ENTRYPOINT ./entrypoint.sh