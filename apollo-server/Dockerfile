FROM node:alpine
WORKDIR ./apollo-server
COPY package.json gateway.js wait-for-it.sh ./
RUN npm install
RUN chmod +x wait-for-it.sh
RUN apk add --update bash
# wait until all downstream services start
CMD ./wait-for-it.sh planets-service:8001 -s -- \
    ./wait-for-it.sh satellites-service:8002 -s -- \
    ./wait-for-it.sh auth-service:8003 -s -- \
    npm run start-gateway
EXPOSE 4000
