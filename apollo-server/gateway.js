const {ApolloServer} = require("apollo-server");
const {ApolloGateway, RemoteGraphQLDataSource} = require("@apollo/gateway");

class AuthenticatedDataSource extends RemoteGraphQLDataSource {
    willSendRequest({request, context}) {
        if (context.authHeaderValue) {
            request.http.headers.set('Authorization', context.authHeaderValue);
        }
    }
}

const gateway = new ApolloGateway({
    serviceList: [
        {name: "planets-service", url: "http://planets-service:8001"},
        {name: "satellites-service", url: "http://satellites-service:8002"},
        {name: "auth-service", url: "http://auth-service:8003"},
    ],
    buildService({name, url}) {
        return new AuthenticatedDataSource({url});
    },
});

const server = new ApolloServer({
    gateway, subscriptions: false, context: ({req}) => ({
        authHeaderValue: req.headers.authorization
    })
});

server.listen({host: "0.0.0.0", port: 4000}).then(({url}) => {
    console.log(`🚀 Server ready at ${url}`);
});
