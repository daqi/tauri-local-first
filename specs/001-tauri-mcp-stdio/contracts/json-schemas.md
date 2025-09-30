# JSON Schemas (Draft)

Pseudo JSON Schema representations (refine in implementation).

## ParsedIntent

```
ParsedIntent = {
  id: string,
  actionName: string,
  targetAppId?: string,
  params: { [k: string]: any },
  confidence: number >=0 <=1,
  sourceTextSpan: { start: number >=0, end: number > start },
  explicit: boolean
}
```

## ExecutionPlan

```
ExecutionPlan = {
  planId: string,
  originalInput: string,
  intents: ParsedIntent[],
  deduplicated: ParsedIntent[],
  batches: ExecutionPlanBatch[],
  conflicts: ConflictDetection[],
  strategy: 'sequential' | 'parallel' | 'mixed',
  generatedAt: integer(timestamp ms),
  dryRun: boolean,
  explain?: ExplainPayload
}
```

## ExecutionPlanBatch

```
ExecutionPlanBatch = {
  batchId: string,
  intents: string[] // intent ids
}
```

## ConflictDetection

```
ConflictDetection = {
  conflictKey: string,
  intents: string[],
  reason: string,
  resolution: 'force-order' | 'user-select' | 'drop-conflicting'
}
```

## ExplainPayload

```
ExplainPayload = {
  tokens: string[],
  matchedRules: { ruleId: string, weight: number, intentId?: string }[]
}
```

## ActionResult

```
ActionResult = {
  intentId: string,
  status: 'success' | 'failed' | 'skipped' | 'timeout' | 'simulated',
  reason?: string,
  retryHint?: string,
  predictedEffects?: string[],
  durationMs?: number,
  startedAt: integer,
  finishedAt?: integer
}
```

## CommandHistoryRecord

```
CommandHistoryRecord = {
  signature: string,
  input: string,
  intentsSummary: string[],
  overallStatus: 'success' | 'partial' | 'failed',
  createdAt: integer,
  planSize: number,
  explainUsed: boolean
}
```

-- END --
