import { Link, useLocation } from 'react-router-dom';
import { Home, PlusCircle, PlayCircle } from 'lucide-react';
import type { ReactNode } from 'react';

export const Layout = ({ children }: { children: ReactNode }) => {
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path ? 'active' : '';

    return (
        <div className="layout">
            <nav className="navbar">
                <Link to="/" className="logo">
                    <span className="logo-text">Neko Words</span> 🐱
                </Link>
                <div className="nav-links">
                    <Link to="/" className={`nav-link ${isActive('/')}`}>
                        <Home size={20} /> <span>Dashboard</span>
                    </Link>
                    <Link to="/add" className={`nav-link ${isActive('/add')}`}>
                        <PlusCircle size={20} /> <span>Add Word</span>
                    </Link>
                    <Link to="/review" className={`nav-link ${isActive('/review')}`}>
                        <PlayCircle size={20} /> <span>Review</span>
                    </Link>
                </div>
                <div style={{ width: 100 }}></div> {/* Spacer for centering if needed */}
            </nav>
            <main className="main-content">
                {children}
            </main>
        </div>
    );
};
