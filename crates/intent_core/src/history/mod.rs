use crate::CommandHistoryRecord;
pub mod sqlite_store;
use std::collections::BTreeMap;
use crate::util::now_ms;

pub trait HistoryStore: Send + Sync {
    fn save(&mut self, record: CommandHistoryRecord);
    fn list(&self, limit: usize, after: Option<u64>) -> Vec<CommandHistoryRecord>;
    fn purge_older_than(&mut self, cutoff_ts: u64) -> usize;
}


pub struct InMemoryHistoryStore {
    // key = created_at (allow duplicates by small increment) ; value = record
    data: BTreeMap<u64, CommandHistoryRecord>,
    retention_ms: u64,
}

impl InMemoryHistoryStore {
    pub fn new(retention_days: u64) -> Self {
        Self {
            data: BTreeMap::new(),
            retention_ms: retention_days * 24 * 60 * 60 * 1000,
        }
    }

    fn adjusted_key(&self, ts: u64) -> u64 {
        // ensure uniqueness by bumping timestamp if collision
        let mut k = ts;
        while self.data.contains_key(&k) {
            k += 1;
        }
        k
    }
}

impl Default for InMemoryHistoryStore {
    fn default() -> Self {
        Self::new(30)
    }
}

impl HistoryStore for InMemoryHistoryStore {
    fn save(&mut self, mut record: CommandHistoryRecord) {
        // purge before inserting
        let cutoff = now_ms().saturating_sub(self.retention_ms);
        self.purge_older_than(cutoff);
        // assign created_at if zero
        if record.created_at == 0 {
            record.created_at = now_ms();
        }
        let key = self.adjusted_key(record.created_at);
        self.data.insert(key, record);
    }

    fn list(&self, limit: usize, after: Option<u64>) -> Vec<CommandHistoryRecord> {
        if limit == 0 {
            return Vec::new();
        }
        let mut keys: Vec<u64> = match after {
            Some(a) => self.data.range((a + 1)..).map(|(k, _)| *k).collect(),
            None => self.data.keys().cloned().collect(),
        };
        // reverse chronological (newest = larger timestamp last inserted)
        keys.sort_unstable();
        let mut out = Vec::new();
        for k in keys.into_iter().rev() {
            if let Some(rec) = self.data.get(&k) {
                out.push(rec.clone());
                if out.len() == limit {
                    break;
                }
            }
        }
        out
    }

    fn purge_older_than(&mut self, cutoff_ts: u64) -> usize {
        let keys: Vec<u64> = self
            .data
            .keys()
            .cloned()
            .filter(|k| *k < cutoff_ts)
            .collect();
        let count = keys.len();
        for k in keys {
            self.data.remove(&k);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandHistoryRecord;

    fn rec(sig: &str, ts: u64) -> CommandHistoryRecord {
        CommandHistoryRecord {
            signature: sig.into(),
            input: format!("input-{sig}"),
            intents_summary: vec!["a".into()],
            overall_status: "success".into(),
            created_at: ts,
            plan_size: 1,
            explain_used: false,
        }
    }

    #[test]
    fn save_and_list_ordering() {
        let mut store = InMemoryHistoryStore::new(30);
        let base = now_ms();
        store.save(rec("s1", base));
        store.save(rec("s2", base + 10));
        let listed = store.list(10, None);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].signature, "s2"); // newest first
        assert_eq!(listed[1].signature, "s1");
    }

    #[test]
    fn pagination_after() {
        let mut store = InMemoryHistoryStore::new(30);
        let base = now_ms();
        store.save(rec("s1", base));
        store.save(rec("s2", base + 10));
        store.save(rec("s3", base + 20));
        // after second timestamp should return only s3
        let after = base + 10;
        let listed = store.list(10, Some(after));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].signature, "s3");
    }

    #[test]
    fn retention_purge() {
        let mut store = InMemoryHistoryStore::new(30);
        let now = now_ms();
        let old = now - (31 * 24 * 60 * 60 * 1000); // 31 days ago
        store.save(rec("old", old));
        store.save(rec("new", now));
        let listed = store.list(10, None);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].signature, "new");
    }
}
