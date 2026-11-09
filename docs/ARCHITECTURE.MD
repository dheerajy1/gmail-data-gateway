I am just pasting that i found necessary, but i want a proper architecture written.

I'm writing the PostgreSQL scripts and the Azure procedure change now, then I'll pack everything.

I'm updating the SPEC and decisions for the Azure id range, then packing everything.

**The `ack-received` warning:** after each ACK the subscriber sends, the gateway replies with an `ack-received` message to confirm it. The subscriber didn't know that message type, so it logged a warning each time. Nothing was wrong. The subscriber now ignores it.

**`azure_id` NULL:** your recordlog commit `b44d904` has the right design, and I applied the same one to jobs. Before this, the Kafka path always inserted NULL and waited for the Azure sync. Now the flow is:
1. The Azure write returns the range of ids it just inserted.
2. The write-model handler adds that range to the event it sends to the read-model topic.
3. The read model passes the range to PostgreSQL.
4. PostgreSQL gives the k-th inserted row `fromId + k`, but only when the range size equals the number of rows it is about to insert. Otherwise it leaves NULL and the Azure sync fills it later, with no failure.

I also made both databases sort ties by `emailId`, so two mails in the same second get the same order on both sides.

**What's attached**
- `jobs-api-gateway-rust-azure-id.tar.gz` has the subscriber changes, the SPEC update and decision D57.
- `db-scripts/` has four SQL files:
  - `azure-02spinsertRawAppliedJobs.sql`
  - `fn_kafka_insert_raw_applied_jobs.sql`
  - `sp_kafka_apply_committed_records.sql`
  - `grant_kafka_pipeline_privileges.sql`

**Two limits of this design**
- A replayed write returns no range, so if its read model runs after that, `azure_id` stays NULL until the sync.
- This is the production Azure procedure. Your development run used the `[sjobsDataDev]` schema, which has its own write procedure that I have not seen.