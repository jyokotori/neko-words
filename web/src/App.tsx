import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { Layout } from './components/Layout';
import { AddWord } from './components/AddWord';
import { Review } from './components/Review';

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<Navigate to="/add" replace />} /> {/* Default to Add for now */}
          <Route path="/add" element={<AddWord />} />
          <Route path="/review" element={<Review />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
