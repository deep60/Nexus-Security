<p align="center">
  <img src="docs/assets/verdyx-banner.png" alt="Verdyx Logo" width="600"/>
</p>

<h3 align="center">Verdyx</h3>
<p align="center">
  <em>Decentralized Threat Intelligence Marketplace</em>
</p>

<p align="center">
  <a href="https://github.com/deep60/Nexus-Security/actions/workflows/rust.yml">
    <img src="https://github.com/deep60/Nexus-Security/actions/workflows/rust.yml/badge.svg" alt="Rust CI">
  </a>
  <a href="https://github.com/deep60/Nexus-Security/actions/workflows/frontend-ci.yml">
    <img src="https://github.com/deep60/Nexus-Security/actions/workflows/frontend-ci.yml/badge.svg" alt="Frontend CI">
  </a>
  <a href="https://github.com/deep60/Nexus-Security/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License">
  </a>
</p>

---

## 🚀 Why Verdyx?  
Traditional antivirus relies on **single-vendor detection**. This creates blind spots, delays in zero-day response, and centralized control.  

**Verdyx flips the model**:  
- 🧑‍💻 **Crowdsourced experts** + 🤖 **automated engines** work together  
- 💰 **Bounty incentives** ensure high-quality analysis  
- ⛓️ **Blockchain transparency** guarantees fair payments & reputation  
- ⚡ **Consensus confidence scores** reduce false positives  

---

## 🛠️ How It Works  
1. A suspicious **file or URL** is submitted with a **bounty**.  
2. Multiple **security engines** (human + automated) analyze it.  
3. Engines **stake tokens** on their verdict (malicious/benign).  
4. **Accurate engines earn**, inaccurate ones **lose stake**.  
5. Verdyx returns a **confidence score** & report.  

---

## ✨ Features  
- 🎯 **Bounty-driven marketplace** for threat analysis  
- ⚡ **Multi-engine detection** (humans + automation)  
- ⛓️ **Ethereum smart contracts** for payments & reputation  
- 🔗 **APIs & integrations** for SOC & SIEM pipelines  
- ⏱️ **Near real-time detection** for new threats  

---

## 📂 Project Structure  

---

## ⚡ Quick Start  

### Prerequisites  
- [Node.js](https://nodejs.org/) >= 18  
- [Rust](https://www.rust-lang.org/)  
- [Python 3.10+](https://www.python.org/)  
- [Docker](https://www.docker.com/)  
- [MetaMask](https://metamask.io/) or Ethereum wallet  

### Setup  
```bash
# Clone repo
git clone https://github.com/your-org/verdyx.git
cd verdyx

# Backend
cd backend
cargo run

# Frontend
cd ../frontend
npm install
npm run dev

# Deploy smart contracts
cd ../smart-contracts
npx hardhat deploy
```
----

## 📖 API Example
``` curl -X POST https://api.verdyx.com/submit \
  -H "Authorization: Bearer <TOKEN>" \
  -F "file=@/path/to/file.exe" \
  -F "bounty=0.05ETH"
```

---

## 🤝 Contributing

We welcome contributions from the community!
See CONTRIBUTING.md
 for guidelines.

## 🔐 Security

If you discover a security vulnerability, please do not create a public issue.
Instead, report it responsibly via security@verdyx.com
.

## 🤝 Join the Community

💬 Discord

🐦 Twitter

📧 Email: security@verdyx.com

📜 License

MIT License © 2025 Verdyx