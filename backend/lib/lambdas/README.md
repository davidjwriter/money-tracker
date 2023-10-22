## Lambda Functions for the Money Tracker

1. **Create Access Token**: This Lambda function is responsible for creating an access token. Is invoked when the user completes the flow using the temporary token and provides this Lambda with the public token. The Lambda then utilizes the Plaid API to establish the link and stores the access_token in our Dynamo DB table.

2. **Get Link**: The first function in the process, its primary function is to establish a link with the Plaid API to obtain a temporary token to be passed back to the client.

3. **Send Weekly Report**: This Lambda Function is called by Event Bridge on a weekly basis. It iterates through the Dynamo DB table and, for all the different access tokens (accounts) available, it identifies all transactions that occurred within the given week. Subsequently, it sends the report to the specified SNS Topic.
