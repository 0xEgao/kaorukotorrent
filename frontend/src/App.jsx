import { useEffect, useMemo, useState } from 'react';

const navActions = [
  { label: 'View Current Market', path: '/market' },
  { label: 'List a Kaoruko Torrent Offer', path: '/sender' },
  { label: 'Receive a Torrent', path: '/receiver' },
];

function parseItemInfo(itemInfo) {
  try {
    return JSON.parse(itemInfo);
  } catch {
    return null;
  }
}

function usePathname() {
  const [pathname, setPathname] = useState(window.location.pathname || '/');

  useEffect(() => {
    const onPop = () => setPathname(window.location.pathname || '/');
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, []);

  const navigate = (path) => {
    if (path === pathname) {
      return;
    }
    window.history.pushState({}, '', path);
    setPathname(path);
  };

  return { pathname, navigate };
}

function apiUrl(scope, path) {
  return `/api/${scope}${path}`;
}

function HomePage({ navigate }) {
  return (
    <section className="home-shell" aria-label="Homepage">
      <img
        src="/kaoruko-home.png"
        alt="Kaoruko homepage art"
        className="home-image"
        onError={(event) => {
          event.currentTarget.style.display = 'none';
          event.currentTarget.nextElementSibling.style.display = 'grid';
        }}
      />
      <div className="image-fallback" aria-hidden="true">
        <span>Drop image at frontend/public/kaoruko-home.png</span>
      </div>
      <div className="home-actions">
        {navActions.map((action) => (
          <button
            key={action.path}
            className="action-btn"
            type="button"
            onClick={() => navigate(action.path)}
          >
            {action.label}
          </button>
        ))}
      </div>
    </section>
  );
}

function TerminalPanel({ title, imageSrc, imageAlt, children, footer }) {
  return (
    <section className="terminal-panel" aria-label={title}>
      <div className="terminal-head">
        <span className="terminal-title">{title}</span>
      </div>
      <div className="terminal-grid">
        <aside className="terminal-side">
          <img
            src={imageSrc}
            alt={imageAlt}
            className="panel-image"
            onError={(event) => {
              event.currentTarget.style.display = 'none';
              event.currentTarget.nextElementSibling.style.display = 'grid';
            }}
          />
          <div className="panel-image-fallback" aria-hidden="true">
            <span>Placeholder</span>
          </div>
        </aside>
        <div className="terminal-body">{children}</div>
        <div className="terminal-foot">{footer}</div>
      </div>
    </section>
  );
}

function MarketPage() {
  const [offers, setOffers] = useState([]);
  const [status, setStatus] = useState('idle');
  const [error, setError] = useState('');

  const refreshOffers = async () => {
    setStatus('loading');
    setError('');
    try {
      const response = await fetch(apiUrl('market', '/offers'));
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `request failed: ${response.status}`);
      }
      const data = await response.json();
      setOffers(Array.isArray(data) ? data : []);
      setStatus('success');
    } catch (err) {
      setStatus('error');
      setError(err.message || 'failed to load market');
    }
  };

  useEffect(() => {
    refreshOffers();
  }, []);

  return (
    <TerminalPanel
      title="Market"
      imageSrc="/kaoruko-market.png"
      imageAlt="Kaoruko market avatar"
      footer="source: market /api/offers"
    >
      <div className="terminal-actions-inline">
        <button type="button" className="action-btn" onClick={refreshOffers}>
          Refresh Market
        </button>
      </div>
      <p className="status-line">status: {status}</p>
      {error && <p className="error-line">error: {error}</p>}
      {offers.length === 0 && status === 'success' ? (
        <p className="muted-line">no active offers</p>
      ) : (
        <div className="table-wrap">
          <table className="terminal-table">
            <thead>
              <tr>
                <th>item</th>
                <th>address</th>
                <th>size(bytes)</th>
                <th>version</th>
              </tr>
            </thead>
            <tbody>
              {offers.map((offer) => (
                <tr key={`${offer.address}-${offer.item}`}>
                  <td>{offer.item}</td>
                  <td className="mono">{offer.address}</td>
                  <td>{offer.item_size}</td>
                  <td>{offer.version}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </TerminalPanel>
  );
}

function SenderPage() {
  const [item, setItem] = useState('ubuntu-iso');
  const [itemPath, setItemPath] = useState('ubuntu-iso.iso');
  const [status, setStatus] = useState('idle');
  const [error, setError] = useState('');
  const [response, setResponse] = useState(null);
  const [latest, setLatest] = useState(null);

  const publishOffer = async (event) => {
    event.preventDefault();
    setStatus('loading');
    setError('');
    try {
      const payload = {
        item,
        item_path: itemPath || null,
      };

      const result = await fetch(apiUrl('sender', '/offer/publish'), {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify(payload),
      });

      if (!result.ok) {
        const text = await result.text();
        throw new Error(text || `request failed: ${result.status}`);
      }

      const json = await result.json();
      setResponse(json);
      setStatus('success');
    } catch (err) {
      setStatus('error');
      setError(err.message || 'publish failed');
    }
  };

  const fetchLatest = async () => {
    setError('');
    try {
      const result = await fetch(apiUrl('sender', '/offer/latest'));
      if (!result.ok) {
        const text = await result.text();
        throw new Error(text || `request failed: ${result.status}`);
      }
      const json = await result.json();
      setLatest(json);
    } catch (err) {
      setError(err.message || 'latest fetch failed');
    }
  };

  return (
    <TerminalPanel
      title="Sender"
      imageSrc="/kaoruko-sender.png"
      imageAlt="Kaoruko sender avatar"
      footer="source: sender /api/offer/publish and /api/offer/latest"
    >
      <form className="terminal-form" onSubmit={publishOffer}>
        <label>
          <span>item</span>
          <input value={item} onChange={(event) => setItem(event.target.value)} />
        </label>
        <label>
          <span>item path</span>
          <input
            value={itemPath}
            onChange={(event) => setItemPath(event.target.value)}
            placeholder="relative path in sender data dir"
          />
        </label>
        <div className="terminal-actions-inline">
          <button className="action-btn" type="submit">
            Publish Offer
          </button>
          <button className="action-btn" type="button" onClick={fetchLatest}>
            Get Latest
          </button>
        </div>
      </form>
      <p className="status-line">status: {status}</p>
      {error && <p className="error-line">error: {error}</p>}
      {response && (
        <pre className="terminal-output">{JSON.stringify(response, null, 2)}</pre>
      )}
      {latest && !response && (
        <pre className="terminal-output">{JSON.stringify(latest, null, 2)}</pre>
      )}
    </TerminalPanel>
  );
}

function ReceiverPage() {
  const [offers, setOffers] = useState([]);
  const [selectedOfferKey, setSelectedOfferKey] = useState('');
  const [status, setStatus] = useState('idle');
  const [error, setError] = useState('');

  const refreshOffers = async () => {
    setStatus('loading');
    setError('');
    try {
      const response = await fetch(apiUrl('market', '/offers'));
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `request failed: ${response.status}`);
      }
      const data = await response.json();
      setOffers(Array.isArray(data) ? data : []);
      setStatus('success');
    } catch (err) {
      setStatus('error');
      setError(err.message || 'receiver failed to list offers');
    }
  };

  useEffect(() => {
    refreshOffers();
  }, []);

  const selectedOffer = offers.find(
    (offer) => `${offer.address}-${offer.item}` === selectedOfferKey
  );

  const metadata = selectedOffer ? parseItemInfo(selectedOffer.item_info) : null;
  const files = metadata?.files ?? [];

  return (
    <TerminalPanel
      title="Receiver"
      imageSrc="/kaoruko-receiver.png"
      imageAlt="Kaoruko receiver avatar"
      footer="source: market offers + sender file endpoints"
    >
      <div className="terminal-actions-inline">
        <button type="button" className="action-btn" onClick={refreshOffers}>
          Scan Offers
        </button>
      </div>
      <p className="status-line">status: {status}</p>
      {error && <p className="error-line">error: {error}</p>}

      <label className="select-wrap">
        <span>offer</span>
        <select
          value={selectedOfferKey}
          onChange={(event) => setSelectedOfferKey(event.target.value)}
        >
          <option value="">select offer...</option>
          {offers.map((offer) => {
            const key = `${offer.address}-${offer.item}`;
            return (
              <option key={key} value={key}>
                {offer.item} @ {offer.address}
              </option>
            );
          })}
        </select>
      </label>

      {!selectedOffer && <p className="muted-line">select an offer to inspect files</p>}

      {selectedOffer && (
        <div className="download-shell">
          <p className="muted-line">item: {selectedOffer.item}</p>
          <p className="muted-line">size: {selectedOffer.item_size} bytes</p>
          {files.length === 0 ? (
            <p className="muted-line">no file metadata found in item_info</p>
          ) : (
            <ul className="file-list">
              {files.map((entry) => {
                const rel = String(entry.path || '').replace(/^\/+/, '');
                const itemRoot = String(selectedOffer.item || '').replace(/\/+$/, '');
                const resource = rel ? `${itemRoot}/${rel}` : itemRoot;
                const href = `${selectedOffer.address.replace(/\/+$/, '')}/api/files/${resource}`;

                return (
                  <li key={entry.path}>
                    <span>{entry.path || selectedOffer.item}</span>
                    <a href={href} target="_blank" rel="noreferrer">
                      fetch
                    </a>
                  </li>
                );
              })}
            </ul>
          )}
        </div>
      )}
    </TerminalPanel>
  );
}

export default function App() {
  const [theme, setTheme] = useState('dark');
  const { pathname, navigate } = usePathname();

  const label = useMemo(
    () => (theme === 'dark' ? 'Light Mode' : 'Dark Mode'),
    [theme]
  );

  const page = useMemo(() => {
    if (pathname === '/market') {
      return <MarketPage />;
    }
    if (pathname === '/sender') {
      return <SenderPage />;
    }
    if (pathname === '/receiver') {
      return <ReceiverPage />;
    }
    return <HomePage navigate={navigate} />;
  }, [navigate, pathname]);

  return (
    <div className={`app ${theme}`}>
      <header className="navbar" role="banner">
        <button className="brand" type="button" onClick={() => navigate('/')}>
          Kaoruko Torrent
        </button>
        <button
          className="theme-toggle"
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
          type="button"
          aria-label="Toggle dark and light mode"
        >
          {label}
        </button>
      </header>

      <main className="content" role="main">
        {page}
      </main>
    </div>
  );
}
