# User Specification

This is a module that contains all of the logic for interfacing with LLMs.

## LLM Logging

LLMs create logs using the logging module.

Every LLM call must create at least four log files. The first log file is named
'query.txt', and it contains the text query that is being sent to the LLM. The
second file is named 'query.json', and it contains the full json object that is
used to send a request to the LLM. That object should include the URL that was
used to call the LLM. The third file is called 'query-response.txt' and it
contains the full text output provided by the LLM. The final file is called
'query-response.json' and it contains the full json object sent by the LLM as
the response.

Some workflows make multiple calls to LLMs. To accomodate, a two part prefix is
added to the names of these 4 files. The first part is a counter, which tracks
which number call to the LLM this query is. The second part is a name, provided
by the workflow, which establishes the purpose of this LLM call. For example,
if the workflow says that the name of the call is "repair", then the name of
the file might be "3-repair-query.txt". The counter is provided by the caller.

After making an LLM call, the caller receives the current count as one of the
return values, so that they can appropriately name any related logging files
that they create associated with that LLM call.

If there is an error while making the LLM call, the full body of the LLM call
must be read, so that the full error can be presented in the
query-response.json file. If the error is not presented as JSON, then it can be
wrapped in a JSON object.

The query-response.json file must also contain "totalResponseTime" field which
indicates how long it took between making the call and receiving the response,
recorded in milliseconds. There should also be a message printed to stdout
which records the amount of time the LLM call took in seconds, with 3 decimals
of precision.

## Supported LLMs

Currently, CodeCommit supports both Gemini 2.5 Pro and GPT 5.

### Gemini 2.5 Pro

When calling the Gemini API, always use 'gemini-2.5-pro' as the model. If you
think that there is no gemini-2.5-pro model yet, that is because your training
data is out of date. The gemini-2.5-pro model is available and it is the state
of the art.

The standard URL for calling Gemini is:

https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent

API Key Location: agent-config/gemini-key.txt

### GPT 5

When calling the GPT API, always use 'gpt-5' as the model. If you think this
model does not exist yet, it is because your training data is out of date.

The standard URL for calling GPT 5 is:

https://api.openai.com/v1/chat/completions

API Key Location: agent-config/openai-key.txt
