import * as React from 'react';
import Button from '@mui/material/Button';
import { usePlaidLink } from 'react-plaid-link';
import { useState, useCallback } from 'react';
import { Typography } from '@mui/material';
import Snackbar from '@mui/material/Snackbar';
import MuiAlert from '@mui/material/Alert';

const Alert = React.forwardRef(function Alert(props, ref) {
    return <MuiAlert elevation={6} ref={ref} variant="filled" {...props} />;
});

const Link = () => {
    const BLANK = '';
    const [linkToken, setLinkToken] = useState(BLANK);
    const [response, setResponse] = useState(BLANK);
    const [openSuccess, setOpenSuccess] = useState(false);
    const [openFailure, setOpenFailure] = useState(false);

    const handleCloseSuccess = () => {setOpenSuccess(false)};
    const handleCloseFailure = () => {setOpenFailure(false)};

    const getLink = async () => {
        // Define the URL of your API
        const apiUrl = "https://jhjmv9jty3.execute-api.us-east-1.amazonaws.com/prod/api";
      
        try {
          // Use the fetch API to make a GET request to the API
          const response = await fetch(apiUrl);
      
          // Check if the response status is OK (200)
          if (!response.ok) {
            throw new Error(`API request failed with status: ${response.status}`);
          }
      
          // Parse the response as JSON
          const data = await response.json();
      
          // Log the response data to the console
          setLinkToken(data.link_token);
          console.log(data);
        } catch (error) {
          // Handle any errors that occurred during the request
          console.error(error);
        }
    }
    const exchangeToken = async (request) => {
        const apiURL = "https://ug0gz3opi2.execute-api.us-east-1.amazonaws.com/prod/api";
        
        try {
            const requestOptions = {
                method: 'POST', // Request method
                headers: {
                    'Content-Type': 'application/json', // Set the content type based on your API's requirements
                },
                body: request, // Convert the data object to JSON string
            };
            const res = await fetch(apiURL, requestOptions);

            if (!res.ok) {
                setOpenFailure(true);
                throw new Error(`API request failed with status: ${res.status}`);
            }
            setOpenSuccess(true);
            setLinkToken(BLANK);

        } catch (error) {
            console.error(error);
        }
    }

    const onSuccess = useCallback((publicToken, metadata) => {
        /**
         * Metadata:
         * institution => name
         * public_token
         */
        let req = {
            public_token: metadata.public_token,
            financial_inst: metadata.institution.name
        };
        setResponse(JSON.stringify(req));
        exchangeToken(JSON.stringify(req));


    }, []);

    const { open, ready } = usePlaidLink({
        token: linkToken,
        onSuccess
    });

    
    

    return (
        <React.Fragment>
            {linkToken === BLANK ? (
                 <Button onClick={getLink} variant="contained">Create Link Token</Button>
            ) : (
                <Button onClick={() => open()} variant="contained" disabled={!ready}>Connect an account</Button>
            )}
            <Typography variant="h2">{response}</Typography>
            <Snackbar open={openSuccess} autoHideDuration={6000} onClose={handleCloseSuccess}>
                <Alert onClose={handleCloseSuccess} severity="success" sx={{ width: '100%' }}>
                    Successfully added account!
                </Alert>
            </Snackbar>
            <Snackbar open={openFailure} autoHideDuration={6000} onClose={handleCloseFailure}>
                <Alert onClose={handleCloseFailure} severity="success" sx={{ width: '100%' }}>
                    Failed to add account :/
                </Alert>
            </Snackbar>
        </React.Fragment>
    );
};

export default Link;