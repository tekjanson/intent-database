Usage notes for Gemini integration

This repository can use the Google Generative Language API (Gemini) when the
`gemini-model-adapter` feature is enabled.

Provide credentials in one of two ways:

- Set the environment variable `GEMINI_KEY` to your API key.
- Or, create a file `secrets/gemini_key` containing your API key (first line is used).

You may also override the endpoint with `GEMINI_ENDPOINT` if you use a proxy or
private deployment.

The server is configured to NOT fallback to another adapter if the Gemini call
fails. Failures are returned to the client as HTTP 502 with a JSON body:

{"error":"adapter_error","message":"<detailed message>"}

Make sure to set credentials before starting the server:

```bash
export GEMINI_KEY="<your-key>"
make start
```
