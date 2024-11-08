use crate::libs::{
    config::Config,
    error::{OurError, OurResult},
    filter::Filter,
};
use imap::{types::Uid, ImapConnection, Session};
use imap_proto::NameAttribute;
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
};

#[derive(Clone, Debug)]
pub struct ListResult<T>
where
    T: Clone + Debug,
{
    pub extra: Option<T>,
}

#[derive(Debug)]
pub struct Imap<T>
where
    T: Clone + Debug + Serialize,
{
    pub session: Session<Box<dyn ImapConnection>>,
    config: Config<T>,
    cached_capabilities: HashMap<String, bool>,
}

impl<T> Drop for Imap<T>
where
    T: Clone + Debug + Serialize,
{
    fn drop(&mut self) {
        if let Err(e) = self.session.logout() {
            eprintln!("error disconnecting: {e}");
        }
    }
}

impl<T> Imap<T>
where
    T: Clone + Debug + Serialize,
{
    /// Connect to the server and login with the given credentials.
    /// # Errors
    /// Many errors can happen
    pub fn connect(config: &Config<T>) -> OurResult<Self> {
        let server = config
            .server
            .as_ref()
            .ok_or_else(|| OurError::config("Missing server"))?;

        let mut client = imap::ClientBuilder::new(server.as_str(), 143).connect()?;

        if config.debug {
            client.debug = true;
        }

        let session = client.login(
            config
                .username
                .as_ref()
                .ok_or_else(|| OurError::config("Missing username"))?,
            config.password()?,
        )?;

        let mut ret = Self {
            session,
            config: config.clone(),
            cached_capabilities: HashMap::new(),
        };

        if !ret.has_capability("UIDPLUS")? {
            return Err(OurError::Uidplus);
        }

        Ok(ret)
    }

    /// Check if the imap server has some capability
    /// # Errors
    /// Imap errors can happen
    pub fn has_capability<S: AsRef<str>>(&mut self, cap: S) -> OurResult<bool> {
        if let Some(&cached_result) = self.cached_capabilities.get(cap.as_ref()) {
            return Ok(cached_result);
        }

        // We can't cache the result of .capabilities() because it returns some
        // strange structure with very limited lifetime, so we ask once each
        // time we need a new capability and cache the result.
        let has_capability = self.session.capabilities()?.has_str(cap.as_ref());

        self.cached_capabilities
            .insert(cap.as_ref().to_string(), has_capability);

        Ok(has_capability)
    }

    /// Get a list of mailboxes given filters, returns a `BTreeMap` so it is
    /// sorted and stable.
    ///
    /// We use a map to be able to have generic filters at the beginning of the
    /// configuration that are overwritten by more specific filters afterwards.
    ///
    /// # Errors
    /// Many errors can happen
    pub fn list(&mut self) -> OurResult<BTreeMap<String, ListResult<T>>> {
        let mut mailboxes: BTreeMap<String, ListResult<T>> = BTreeMap::new();

        for filter in self.config.filters.clone().unwrap_or_else(||
            // If we don't have a filter, provide an empty one matching everything
            vec![Filter::default()])
        {
            let mut found = false;

            for mailbox in self
                .session
                .list(filter.reference.as_deref(), filter.pattern.as_deref())
                .map_err(|e| OurError::config(format!("Imap error {e} for {filter:?}")))?
                .iter()
                // Filter out folders that are marked as NoSelect, which are not mailboxes, only folders
                .filter(|mbx| !mbx.attributes().contains(&NameAttribute::NoSelect))
                // If we have an include regex, keep folders that match it
                // Otherwise, keep everything
                .filter(|mbx| {
                    filter
                        .include_re
                        .as_ref()
                        .map_or(true, |re| re.is_match(mbx.name()))
                })
                // If we have an exclude regex, filter out folders that match it
                // Otherwise, keep everything
                .filter(|mbx| {
                    filter
                        .exclude_re
                        .as_ref()
                        .map_or(true, |re| !re.is_match(mbx.name()))
                })
            {
                found = true;
                mailboxes.insert(
                    mailbox.name().to_string(),
                    ListResult {
                        extra: filter.extra.clone().or_else(|| self.config.extra.clone()),
                    },
                );
            }
            if !found {
                return Err(OurError::config(format!(
                    "This filter did not return anything {filter:?}"
                )));
            }
        }

        Ok(mailboxes)
    }
}

pub fn ids_list_to_collapsed_sequence(ids: &HashSet<Uid>) -> String {
    if ids.is_empty() {
        todo!("nothing in there"); // TODO: do something ?
    }

    // Collect and sort the IDs
    let mut sorted_ids: Vec<Uid> = ids.iter().copied().collect();
    sorted_ids.sort_unstable();

    // Collect ranges from the sorted list
    let mut result = Vec::new();
    let mut start = sorted_ids.first().copied();
    let mut end = start;

    for &id in &sorted_ids[1..] {
        match (end, start) {
            (Some(e), Some(_s)) if id == e + 1 => end = Some(id),
            _ => {
                // Push the previous range
                if let (Some(s), Some(e)) = (start, end) {
                    result.push(if s == e {
                        s.to_string()
                    } else {
                        format!("{s}:{e}")
                    });
                }
                start = Some(id);
                end = start;
            }
        }
    }

    // Push the last range
    if let (Some(s), Some(e)) = (start, end) {
        result.push(if s == e {
            s.to_string()
        } else {
            format!("{s}:{e}")
        });
    }

    result.join(",")
}

#[cfg(test)]
mod tests {
    use super::ids_list_to_collapsed_sequence;
    use imap::types::Uid;
    use std::collections::HashSet; // Assuming this function is in a module named 'ids_list_to_collapsed_sequence'

    #[test]
    #[should_panic(expected = "not yet implemented: nothing in there")]
    fn test_empty_set() {
        let ids: HashSet<Uid> = HashSet::new();
        // Assuming `ids_list_to_collapsed_sequence` returns an empty string for an empty set
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "");
    }

    #[test]
    fn test_single_id() {
        let mut ids = HashSet::new();
        ids.insert(5);
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "5");
    }

    #[test]
    fn test_continuous_range() {
        let ids: HashSet<Uid> = [1, 2, 3, 4, 5].iter().copied().collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1:5");
    }

    #[test]
    fn test_multiple_disjoint_ranges() {
        let ids: HashSet<Uid> = [1, 2, 3, 7, 8, 10, 11].iter().copied().collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1:3,7:8,10:11");
    }

    #[test]
    fn test_mixed_ranges_and_single_ids() {
        let ids: HashSet<Uid> = [1, 3, 4, 6, 7, 10, 12].iter().copied().collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1,3:4,6:7,10,12");
    }

    #[test]
    fn test_unsorted_input() {
        let ids: HashSet<Uid> = [10, 1, 4, 5, 12, 6, 22, 23, 24, 31]
            .iter()
            .copied()
            .collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1,4:6,10,12,22:24,31");
    }
}
