# Welcome to the Weekly Money Tracker

This is a project that uses AWS and Plaid to establish links to all financial institutions and send weekly budget reports.

1. Call /link/token/create to create a link_token and pass the temporary token to your app's client.
2. Use the link_token to open Link for your user. In the onSuccess callback, Link will provide a temporary public_token.
3. Call /item/public_token/exchange to exchange the public_token for a permanent access_token and item_id for the new Item.
4. Store the access_token and use it to make product requests for your user's Item.
