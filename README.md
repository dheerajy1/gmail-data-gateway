# Gmail-data-gateway
Gmail data gateway

# API Documentation Examples

## Bulk
Behavior:

1. Pull everything

2. Full pagination

3. Ignores DB

```JSON
{
  "type": "Bulk",
  "query": "label:job's-mails-applied-jobs"
}
```

## Sync (Production Cron Job)

Behavior:

1. Gets MAX(date) from DB

2. Builds after:lastDate

3. Fetches only new mails

4. Safe to call repeatedly

5. Idempotent

Note: 

1. Uses DB max(date).

```JSON
{
  "type": "Sync",
  "query": "label:job's-mails-applied-jobs"
}
```

## Increment (Manual Window)

```JSON
{
  "type": "Increment",
  "query": "label:job's-mails-applied-jobs",
  "inputDateAfter": "2026-03-01T00:00:00+05:30"
}
```

## Increment With Range

```JSON
{
  "type": "Increment",
  "query": "label:job's-mails-applied-jobs",
  "inputDateAfter": "2026-03-01T00:00:00+05:30",
  "inputDateBefore": "2026-03-02T00:00:00+05:30"
}
```
