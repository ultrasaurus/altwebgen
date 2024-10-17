# getting started: https://docs.docker.com/get-started/workshop/02_our_app/
FROM ubuntu:latest
WORKDIR /app
COPY . .
# RUN yarn install --production
# CMD ["node", "src/index.js"]
# EXPOSE 3000