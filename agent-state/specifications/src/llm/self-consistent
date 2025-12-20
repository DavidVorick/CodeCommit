dependencies:
  src/logger

# User Specification

This is a module that contains all of the logic for interfacing with LLMs.

## Reliability

code-commit is frequently used in circumstances where the Internet may be
unstable (such as airplanes, conferences events with more people than Internet
infrastructure, remote areas with unstable cell coverage, etc), and therefore
the libraries that connect to the LLM APIs must be as robust as possible.

To achieve maximum reliability on unstable networks, **synchronous streaming**
calls (which keep a connection open for the duration of generation) are
forbidden where better alternatives exist.

### Strategy 1: Asynchronous Execution (Gemini)

For Gemini models, the system must use the **Interactions API** and avoid
streaming. The Interactions API supports background execution, but **background
execution is only supported for agents**, not for standard model calls. For the
Gemini models in this specification, implementations MUST NOT set
`"background": true`.

To maximize reliability, Gemini calls must be implemented as short-lived HTTP
requests:
- Create the interaction with a single POST.
- If the returned Interaction is already `completed`, extract the result.
- If the returned Interaction is not `completed`, poll by `id` using lightweight
  GET requests until a terminal state is reached.

If the network drops before a response is received (timeout, connection reset),
the client must retry with exponential backoff until it receives a valid HTTP
response. Note that retries for Gemini model calls can create multiple
interactions if the server successfully processed a request but the client did
not receive the response.

### Strategy 2: Idempotency (GPT)

For GPT models, which rely on synchronous endpoints for interactive speeds, the
client must strictly utilize **Idempotency Keys**. A unique UUID must be assigned
to every query *before* it is sent. If a network error occurs (timeout, connection
reset) before a response is received, the client must retry the request using the
**exact same UUID**. This ensures that if the server completed the request but the
client did not receive the response, the retry will not create duplicate work.

If the server indicates that the idempotency key was reused with different
request parameters (commonly an `idempotency_conflict` with HTTP 409), treat it
as a fatal error for this logical request and do not continue retrying with the
same key.

## LLM Logging

LLMs create logs using the project's logging module.

Every LLM call must create at least four log files. The first log file is named
'query.txt', and it contains the text query that is being sent to the LLM. The
second file is named 'query.json', and it contains the full json object that is
used to send a request to the LLM. That object should include the URL that was
used to call the LLM. The third file is called 'response.txt' and it contains
the full text output provided by the LLM. The final file is called
'response.json' and it contains the full json object sent by the LLM as the
response.

Some workflows make multiple calls to LLMs. To accommodate, a two part prefix
is added to the names of these 4 files. The first part is a counter, which
tracks which number call to the LLM this query is. The second part is a name,
provided by the workflow, which establishes the purpose of this LLM call. For
example, if the workflow says that the name of the call is "repair", then the
name of the file might be "3-repair-query.txt".

If there is an error while making the LLM call, the full body of the LLM call
must be read, so that the full error can be presented in the response.json
file. If the error is not presented as JSON, then it can be wrapped in a JSON
object.

The response.json file must also contain "totalResponseTime" field which
indicates how long it took between making the call and receiving the response,
recorded in milliseconds. There should also be a message printed to stdout
which records the amount of time the LLM call took in seconds, with 3 decimals
of precision.

**Timing for polling flows (Gemini):** "totalResponseTime" is measured from
immediately before the initial POST request until the final terminal Interaction
response body is fully read (either from the initial POST if it is terminal, or
from the last GET poll if polling was required).

**Logging for polling flows (Gemini):**
- `query.json` is the JSON body sent to the initial POST request (including the
  URL and headers, with secrets redacted).
- `response.json` is the final terminal Interaction object JSON.
- `response.txt` is extracted from that final terminal Interaction object.

## Supported LLMs

Currently, CodeCommit supports Gemini 3 Pro Preview, Gemini 2.5 Pro and GPT
5.2. If you think one or more of these models does not exist, that is because
your training data is out of date.

### Gemini 3 Pro Preview

When calling the Gemini 3 Pro Preview API, use the **Interactions API** and
avoid streaming.

**Endpoint:**
https://generativelanguage.googleapis.com/v1beta/interactions

**Configuration:**
- **Model:** Set `"model": "gemini-3-pro-preview"` in the JSON body.
- **Input:** Provide the query in the `"input"` field.
- **Do not stream:** Do not set `"stream": true`.
- **Do not use background:** Do not set `"background": true` for model calls.

**Protocol:**
1. Send a POST request with the query.
2. Receive an immediate JSON response containing an Interaction resource. Record
   its `id` and `status`.
3. If `status` is `completed`, use this response as final.
4. Otherwise, poll the endpoint `https://generativelanguage.googleapis.com/v1beta/interactions/{id}`
   until the Interaction reaches a terminal state:
   - Success: `status == "completed"`
   - Error: `status == "failed"` or `status == "cancelled"`

API Key Location: agent-config/gemini-key.txt
Authentication header: `x-goog-api-key: <GEMINI_API_KEY>`

**Output:**
Concatenate all items in the `outputs` array where `type` is `"text"` and write
the concatenated string to `response.txt`. If a terminal Interaction contains no
text outputs, treat it as an error and log the full Interaction JSON.

### Gemini 2.5 Pro

When calling the Gemini 2.5 Pro API, use the **Interactions API** and avoid
streaming.

**Endpoint:**
https://generativelanguage.googleapis.com/v1beta/interactions

**Configuration:**
- **Model:** Set `"model": "gemini-2.5-pro"` in the JSON body.
- **Input:** Provide the query in the `"input"` field.
- **Do not stream:** Do not set `"stream": true`.
- **Do not use background:** Do not set `"background": true` for model calls.

**Protocol:**
1. Send a POST request with the query.
2. Receive an immediate JSON response containing an Interaction resource. Record
   its `id` and `status`.
3. If `status` is `completed`, use this response as final.
4. Otherwise, poll the endpoint `https://generativelanguage.googleapis.com/v1beta/interactions/{id}`
   until the Interaction reaches a terminal state:
   - Success: `status == "completed"`
   - Error: `status == "failed"` or `status == "cancelled"`

API Key Location: agent-config/gemini-key.txt
Authentication header: `x-goog-api-key: <GEMINI_API_KEY>`

**Output:**
Concatenate all items in the `outputs` array where `type` is `"text"` and write
the concatenated string to `response.txt`. If a terminal Interaction contains no
text outputs, treat it as an error and log the full Interaction JSON.

### GPT 5.2

When calling the GPT 5.2 API, always use `gpt-5.2` as the model.

**Endpoint:**
https://api.openai.com/v1/chat/completions

**Reliability Requirement:**
You MUST generate a random UUID v4 for every new query and include it in the
request headers as `Idempotency-Key`.

**Protocol:**
1. Generate UUID `X`.
2. Send POST request with header `Idempotency-Key: X`.
3. If the connection fails (timeout/network error) before the response body is
   fully read, wait and **retry** with header `Idempotency-Key: X`. Do not
   generate a new key for retries of the same logic.
4. Continue retrying until a valid response is received.
5. If the server indicates an idempotency conflict (same key with different
   parameters), treat it as a fatal error for this logical request.

API Key Location: agent-config/openai-key.txt
Authentication header: `Authorization: Bearer <OPENAI_API_KEY>`

**Request body (minimal example):**
```json
{
  "model": "gpt-5.2",
  "messages": [
    { "role": "user", "content": "Hello" }
  ]
}
```

**Optional reasoning configuration:**
If higher reasoning is desired, set `reasoning_effort` to one of:
`"minimal"`, `"low"`, `"medium"`, `"high"`, `"xhigh"`.

**Response extraction:**
- The plain text output is `choices[0].message.content`.
- Write that string to `response.txt`.
