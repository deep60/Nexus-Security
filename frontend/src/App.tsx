import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import Layout from './components/ui/Layout';
import Dashboard from './pages/Dashboard';
import Analysis from './pages/Analysis';
import Bounties from './pages/Bounties';
import Profile from './pages/Profile';
import ExpertHub from './pages/ExpertHub';
import Marketplace from './pages/Marketplace';
import Community from './pages/Community';
import { Web3Provider } from './contexts/Web3Context';
import { ApiProvider } from './contexts/ApiContext';
import './App.css';

function App() {
  return (
    <Web3Provider>
      <ApiProvider>
        <Router>
          <div className="App bg-gray-900 min-h-screen text-white">
            <Layout>
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/analysis" element={<Analysis />} />
                <Route path="/bounties" element={<Bounties />} />
                <Route path="/expert-hub" element={<ExpertHub />} />
                <Route path="/marketplace" element={<Marketplace />} />
                <Route path="/community" element={<Community />} />
                <Route path="/profile" element={<Profile />} />
              </Routes>
            </Layout>
            <Toaster
              position="top-right"
              toastOptions={{
                duration: 4000,
                style: {
                  background: '#1f2937',
                  color: '#fff',
                  border: '1px solid #374151',
                },
                success: {
                  style: {
                    border: '1px solid #10b981',
                  },
                },
                error: {
                  style: {
                    border: '1px solid #ef4444',
                  },
                },
              }}
            />
          </div>
        </Router>
      </ApiProvider>
    </Web3Provider>
  );
}

export default App;