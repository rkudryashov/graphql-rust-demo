const {ApolloServer} = require("apollo-server");
const {ApolloGateway, RemoteGraphQLDataSource} = require("@apollo/gateway");

class AuthenticatedDataSource extends RemoteGraphQLDataSource {
    willSendRequest({request, context}) {
        request.http.headers.set('Authorization', context.authHeaderValue);
    }
}

const gateway = new ApolloGateway({
    serviceList: [
        {name: "planets-service", url: "http://localhost:8001"},
        {name: "satellites-service", url: "http://localhost:8002"}
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

server.listen().then(({url}) => {
    console.log(`🚀 Server ready at ${url}`);
});
