FROM node:alpine
WORKDIR /usr/apollo-server
COPY package.json gateway.js wait-for-it.sh ./
RUN npm install
RUN chmod +x wait-for-it.sh
RUN apk add --update bash
# wait for all downstream services
CMD ./wait-for-it.sh planets-service:8001 -s -- \
    ./wait-for-it.sh satellites-service:8002 -s -- \
    ./wait-for-it.sh auth-service:8003 -s -- \
    npm run start-gateway
