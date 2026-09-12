# Skill: Track Application

When the user wants to update a job application's status or view their application pipeline, manage the job tracking metadata.

## Status Updates

When the user says things like "I applied to Google", "Got a rejection from Stripe", "Interview scheduled with Meta":

### 1. Identify the branch
- Match company/role to an existing branch name
- Run `rvm branch` to list branches if needed
- Ask the user which branch if ambiguous

### 2. Update job.json
Read `.rvm/branches/<branch>/job.json` and update:

```json
{
  "status": "<new_status>",
  "applied_date": "<set when status becomes 'applied'>",
  "events": [
    ...existing_events,
    {
      "status": "<new_status>",
      "timestamp": "<now ISO8601>",
      "notes": "<any notes from user>"
    }
  ]
}
```

### Valid statuses (in order)
| Status | Trigger phrases |
|--------|----------------|
| `draft` | "started", "created branch" |
| `applied` | "applied", "submitted", "sent application" |
| `screening` | "phone screen", "recruiter call", "HR screening" |
| `interviewing` | "interview", "technical round", "onsite" |
| `offered` | "got an offer", "offer letter" |
| `accepted` | "accepted the offer", "signed" |
| `rejected` | "rejected", "didn't get it", "passed on me" |
| `ghosted` | "no response", "ghosted", "haven't heard back in weeks" |
| `withdrawn` | "withdrew", "pulled out", "not interested anymore" |

### 3. Set platform and contacts
When the user mentions where they applied or who they talked to:

**Platform** (add to job.json `notes` or a custom field):
- "Applied on LinkedIn" -> platform: linkedin
- "Found on Indeed" -> platform: indeed
- "Company careers page" -> platform: company_website
- "Got a referral from John" -> platform: referral

**Contacts** (add to job.json `contacts` array):
```json
{
  "name": "Jane Smith",
  "role": "Recruiter",
  "email": "jane@company.com",
  "notes": "Initial phone screen scheduled for Friday"
}
```

### 4. Confirm and show status
After updating, show:
- Branch name and company/role
- Current status
- Full timeline of events
- Next suggested action

## View Pipeline

When user asks "show my applications", "what's my pipeline", "status of all jobs":

```bash
rvm status
```

Then provide a clean summary grouped by status:
- Active (applied/screening/interviewing)
- Pending (draft)
- Closed (offered/accepted/rejected/ghosted/withdrawn)

## Archive Completed Applications

When an application reaches a terminal state (accepted/rejected/ghosted/withdrawn) and the user wants to clean up:

```bash
rvm archive <branch>
```

## Example Interactions

User: "I just submitted my application to the Google SWE role on LinkedIn"
-> Update google-swe job.json: status=applied, applied_date=now, platform=linkedin

User: "Got a call from Sarah at Google for a phone screen next Tuesday"
-> Update google-swe: status=screening, add contact Sarah, add event with notes

User: "Rejected from Stripe"
-> Update stripe-backend: status=rejected, add event
-> Suggest: "Want to archive the stripe-backend branch?"

User: "Show me all my applications"
-> Run `rvm status`, format nicely with statuses and dates
