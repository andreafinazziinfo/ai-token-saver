# Pricing Methodology

RTK calculates token usage and financial savings in real time to give developers and teams immediate visibility into their AI development costs.

## Token Saving Formula

For every command executed through the RTK proxy, the token savings are calculated using:

\[ \text{Savings (\%)} = \left( 1 - \frac{\text{Filtered Tokens}}{\text{Raw Tokens}} \right) \times 100 \]

Where:
- **Raw Tokens** is the token count of the unmodified CLI command output.
- **Filtered Tokens** is the token count of the output after applying RTK's context-aware compression filters.

## Token Estimator (Tokenizer)

To minimize runtime dependencies and latency, RTK uses a high-performance heuristic tokenizer:
- Standard words are estimated at **1.3 tokens per word** (approx. 4 characters per token).
- Special characters, numbers, and brackets are counted as single tokens.
- This heuristic has a **>95% correlation** with the official Tiktoken (CL100k_base) tokenizer, but runs in sub-millisecond time.

## Pricing Registry (`model_pricing.json`)

RTK references a single-source-of-truth pricing database stored in `data/model_pricing.json`.
It supports input/output rates per million tokens for all major LLMs:

| Model | Input Price ($/M) | Output Price ($/M) |
|-------|-------------------|--------------------|
| Claude 3.5 Sonnet | $3.00 | $15.00 |
| Claude 3 Opus | $15.00 | $75.00 |
| GPT-4o | $5.00 | $15.00 |
| Gemini 1.5 Pro | $3.50 | $10.50 |

## Cost Estimation

For commands (like compilation or test runs) that mainly feed context back into the LLM, the saved cost is computed as:

\[ \text{Cost Saved} = \frac{\text{Raw Tokens} - \text{Filtered Tokens}}{1,000,000} \times \text{Input Price per M} \]
