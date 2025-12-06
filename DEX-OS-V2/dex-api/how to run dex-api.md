Summary: How to Run dex-api Properly
Based on our investigation, here's what you need to do to run the dex-api properly:
Prerequisites:
PostgreSQL Database: The dex-api requires a PostgreSQL database to run. It's not compatible with SQLite as currently implemented.
Steps to Run:
Install PostgreSQL if you don't have it already.
Create a database for the DEX API:
sql
CREATE DATABASE dex_api;
Set environment variables and run the API:
powershell
cd D:\DEX-OS-V2\DEX-OS-V2
$env:DATABASE_URL = "postgresql://localhost:5432/dex_api"
$env:JWT_SECRET = "f4a7bd85e0e2b9d8c61276f4c3aa2b908e6fd8a7b0428036b89a9e6e54f9d9a5"
cargo run -p dex-api --release --bin dex-api
Expected Output:
If everything is set up correctly, you should see:
plaintext
Starting DEX-OS API server on port 3030
This indicates that the API server is running and listening on port 3030.
Test the API:
Once running, you can test the API with curl or any HTTP client:
powershell
curl http://localhost:3030/health
Regarding Tests:
The tests are currently failing due to missing imports in the test module. The main issues are:
Missing imports for routes, ApiState, ChallengeStore, Config, ApiCreateProposalResponse, and ApiProposal
Missing imports for GlobalDAO from dex_core::governance

```
running the dex-api: 
 $env:JWT_SECRET="f4a7bd85e0e2b9d8c61276f4c3aa2b908e6fd8a7b0428036b89a9e6e54f9d9a5"; $env:DATABASE_URL="postgres://user:password@localhost:5432/dummy"; cargo run -p dex-api --bin dex-api

```