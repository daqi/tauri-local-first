# Tauri Command Contracts

All commands return JSON serialized structures. Errors use a unified error envelope:

```
{
  "error": { "code": string, "message": string }
}
```

Success responses exclude the `error` key.

## parse_intent

Request:

```
{
  "input": string,
  "options"?: { "explain"?: boolean }
}
```

Response:

```
{
  "planId": string,
  "intents": ParsedIntent[],
  "deduplicated": ParsedIntent[],
  "conflicts": ConflictDetection[],
  "explain"?: ExplainPayload
}
```

Errors:

- INPUT_EMPTY
- PARSE_FAILED

## dry_run

Request:

```
{
  "input": string,
  "options"?: { "explain"?: boolean }
}
```

Response:

```
{
  "plan": ExecutionPlan,            // dryRun=true
  "results": ActionResult[]         // all status=simulated
}
```

Errors:

- INPUT_EMPTY
- PLAN_CONFLICT_UNRESOLVED (if requires user selection)

## execute_plan

Request:

```
{
  "planId"?: string,      // previously produced OR
  "input"?: string,       // raw input (mutually exclusive)
  "options"?: { "explain"?: boolean }
}
```

Validation: Exactly one of planId or input must be provided. Response:

```
{
  "plan": ExecutionPlan,         // dryRun=false
  "results": ActionResult[],
  "overallStatus": "success" | "partial" | "failed"
}
```

Errors:

- INPUT_EMPTY
- PLAN_NOT_FOUND
- PLAN_CONFLICT_UNRESOLVED
- EXECUTION_TIMEOUT (global fail if all actions time out)

## list_history

Request:

```
{
  "limit"?: number,       // default 20
  "after"?: number        // timestamp cursor
}
```

Response:

```
{
  "items": CommandHistoryRecord[],
  "nextCursor"?: number
}
```

Errors:

- INVALID_PAGINATION

## Schema Notes

- `ParsedIntent.confidence` may be omitted in UI if explicit=true.
- `ActionResult.predictedEffects` only present when status=simulated.
- Conflict requiring user decision: return `PLAN_CONFLICT_UNRESOLVED` and include partial plan + conflicts.

-- END --
