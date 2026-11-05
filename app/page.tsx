"use client";

import { FormEvent, useMemo, useState } from "react";

type Nft = {
  id: number;
  name: string;
  creator: string;
  collection: string;
  price: number;
  art: string;
  accent: string;
  category: string;
  listed: boolean;
};
const seed: Nft[] = [
  {
    id: 1,
    name: "Quiet Orbit #014",
    creator: "Studio Aster",
    collection: "Quiet Orbit",
    price: 2.4,
    art: "◉",
    accent: "violet",
    category: "Abstract",
    listed: true,
  },
  {
    id: 2,
    name: "Garden After Rain",
    creator: "Mira Sol",
    collection: "Soft Landscapes",
    price: 1.15,
    art: "❋",
    accent: "green",
    category: "Nature",
    listed: true,
  },
  {
    id: 3,
    name: "Small Signal 03",
    creator: "Field Objects",
    collection: "Small Signals",
    price: 3.8,
    art: "⌁",
    accent: "blue",
    category: "Abstract",
    listed: true,
  },
  {
    id: 4,
    name: "Sunroom Study",
    creator: "Noah Vale",
    collection: "Soft Landscapes",
    price: 0.92,
    art: "◒",
    accent: "orange",
    category: "Nature",
    listed: true,
  },
  {
    id: 5,
    name: "New Form / 21",
    creator: "Studio Aster",
    collection: "Quiet Orbit",
    price: 5.25,
    art: "◇",
    accent: "pink",
    category: "Generative",
    listed: true,
  },
  {
    id: 6,
    name: "A Place to Begin",
    creator: "Eli North",
    collection: "Field Notes",
    price: 1.78,
    art: "△",
    accent: "gold",
    category: "Illustration",
    listed: true,
  },
  {
    id: 7,
    name: "Tidal Memory",
    creator: "Mira Sol",
    collection: "Soft Landscapes",
    price: 2.05,
    art: "≋",
    accent: "teal",
    category: "Nature",
    listed: true,
  },
  {
    id: 8,
    name: "Quiet Orbit #022",
    creator: "Studio Aster",
    collection: "Quiet Orbit",
    price: 4.1,
    art: "⊙",
    accent: "violet",
    category: "Abstract",
    listed: true,
  },
];
const generatedArt: Nft[] = Array.from({ length: 22 }, (_, index) => {
  const collection = ["Quiet Orbit", "Soft Landscapes", "Field Notes", "Small Signals"][index % 4];
  const creator = ["Studio Aster", "Mira Sol", "Eli North", "Field Objects"][index % 4];
  const category = ["Abstract", "Nature", "Illustration", "Generative"][index % 4];
  const number = String(index + 23).padStart(3, "0");
  return {
    id: index + 9,
    name: `${collection} Study #${number}`,
    creator,
    collection,
    price: Number((0.75 + ((index * 17) % 425) / 100).toFixed(2)),
    art: ["◉", "❋", "⌁", "◇"][index % 4],
    accent: ["violet", "green", "blue", "orange"][index % 4],
    category,
    listed: true,
  };
});

export default function SolMintPage() {
  const [items, setItems] = useState([...seed, ...generatedArt]);
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState("All art");
  const [sort, setSort] = useState("Recently listed");
  const [tab, setTab] = useState("Explore");
  const [selected, setSelected] = useState<Nft | null>(null);
  const [mintOpen, setMintOpen] = useState(false);
  const [mintName, setMintName] = useState("");
  const [mintDescription, setMintDescription] = useState("");
  const [fileName, setFileName] = useState("");
  const [favorites, setFavorites] = useState<number[]>([]);
  const [notice, setNotice] = useState("");
  const shown = useMemo(() => {
    const list = items.filter(
      (item) =>
        (tab !== "Favorites" || favorites.includes(item.id)) &&
        (category === "All art" || item.category === category) &&
        `${item.name} ${item.creator} ${item.collection}`
          .toLowerCase()
          .includes(search.toLowerCase()),
    );
    return sort === "Price: low to high"
      ? [...list].sort((a, b) => a.price - b.price)
      : sort === "Price: high to low"
        ? [...list].sort((a, b) => b.price - a.price)
        : list;
  }, [items, tab, favorites, category, search, sort]);
  const createPreview = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const name = mintName.trim();
    if (!name) return;
    const item: Nft = {
      id: Date.now(),
      name,
      creator: "You · local preview",
      collection: "Unminted previews",
      price: 0,
      art: "✳",
      accent: "pink",
      category: "Generative",
      listed: false,
    };
    setItems((current) => [item, ...current]);
    setMintOpen(false);
    setMintName("");
    setMintDescription("");
    setFileName("");
    setNotice("Preview created locally. Nothing was minted or listed on-chain.");
    setTab("My previews");
  };
  const toggleFavorite = (id: number) =>
    setFavorites((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    );
  const selectTab = (next: string) => {
    setTab(next);
    if (next === "My previews") setCategory("All art");
  };

  return (
    <main className="mint-app">
      <header className="mint-header">
        <a className="mint-brand" href="#top">
          <span>✳</span> SOLMINT
        </a>
        <nav>
          <button
            className={tab === "Explore" ? "nav-active" : ""}
            onClick={() => selectTab("Explore")}
          >
            Explore
          </button>
          <button
            className={tab === "Collections" ? "nav-active" : ""}
            onClick={() => selectTab("Collections")}
          >
            Collections
          </button>
          <button
            className={tab === "Favorites" ? "nav-active" : ""}
            onClick={() => selectTab("Favorites")}
          >
            Favorites
          </button>
        </nav>
        <label className="mint-search">
          ⌕
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search art & creators"
          />
        </label>
        <button
          className="connect-button"
          onClick={() =>
            setNotice(
              "Wallet connection is disabled in this showcase. No wallet or network is connected.",
            )
          }
        >
          Connect wallet <span>↗</span>
        </button>
      </header>
      <div className="chain-disclosure">
        <span>LOCAL MARKETPLACE PREVIEW</span> Sample artwork and prices only · Wallet and Solana
        network are not connected
      </div>
      <section className="mint-hero" id="top">
        <div>
          <span className="mint-eyebrow">DIGITAL ART / INDEPENDENT CREATORS</span>
          <h1>
            Collect what
            <br />
            <em>moves you.</em>
          </h1>
          <p>Explore a considered collection of digital art from independent sample creators.</p>
          <button
            onClick={() =>
              document.querySelector(".market-section")?.scrollIntoView({ behavior: "smooth" })
            }
          >
            Browse the collection <span>↓</span>
          </button>
        </div>
        <div className="hero-art" aria-hidden="true">
          <div className="hero-orbit orbit-a" />
          <div className="hero-orbit orbit-b" />
          <div className="hero-sun">✳</div>
          <span>ART / 001</span>
        </div>
      </section>
      <section className="collection-strip">
        <div>
          <span className="collection-icon">◉</span>
          <span>
            <small>CURATED SERIES</small>
            <b>Quiet forms, open spaces</b>
          </span>
        </div>
        <span>
          08 <small>sample pieces</small>
        </span>
        <span>
          Illustrative floor <b>0.92 SOL</b>
        </span>
        <button
          onClick={() => {
            selectTab("Collections");
            setCategory("All art");
            document.querySelector(".market-section")?.scrollIntoView({ behavior: "smooth" });
          }}
        >
          Explore series ↗
        </button>
      </section>
      <section className="market-section">
        <div className="market-heading">
          <div>
            <span className="mint-eyebrow">THE MARKETPLACE / SAMPLE COLLECTION</span>
            <h2>
              {tab === "Explore"
                ? "A little art for your day."
                : tab === "My previews"
                  ? "Your local previews."
                  : `${tab}, thoughtfully chosen.`}
            </h2>
          </div>
          <button className="create-button" onClick={() => setMintOpen(true)}>
            ＋ <span>Create preview</span>
          </button>
        </div>
        <div className="market-tools">
          <div className="category-tabs">
            {["All art", "Abstract", "Nature", "Generative", "Illustration"].map((item) => (
              <button
                key={item}
                className={category === item ? "category-active" : ""}
                onClick={() => setCategory(item)}
              >
                {item}
              </button>
            ))}
          </div>
          <label className="sort-select">
            Sort by{" "}
            <select value={sort} onChange={(event) => setSort(event.target.value)}>
              <option>Recently listed</option>
              <option>Price: low to high</option>
              <option>Price: high to low</option>
            </select>
          </label>
        </div>
        <div className="nft-grid">
          {shown
            .filter((item) => tab !== "My previews" || !item.listed)
            .filter((item) => tab !== "Collections" || item.collection === "Quiet Orbit")
            .map((item, index) => (
              <article className="nft-card" key={item.id}>
                <button
                  className={`nft-art art-${item.accent}`}
                  onClick={() => setSelected(item)}
                  aria-label={`View ${item.name}`}
                >
                  <span>{item.art}</span>
                  <small>{item.collection.toUpperCase()}</small>
                  <b>0{index + 1}</b>
                </button>
                <div className="nft-info">
                  <div>
                    <button className="nft-name" onClick={() => setSelected(item)}>
                      {item.name}
                    </button>
                    <small>by {item.creator}</small>
                  </div>
                  <button
                    className={`favorite ${favorites.includes(item.id) ? "favorite-on" : ""}`}
                    onClick={() => toggleFavorite(item.id)}
                    aria-label={
                      favorites.includes(item.id) ? "Remove from favorites" : "Add to favorites"
                    }
                  >
                    {favorites.includes(item.id) ? "♥" : "♡"}
                  </button>
                </div>
                <div className="nft-price">
                  <span>{item.listed ? "CURRENT PRICE" : "LOCAL PREVIEW"}</span>
                  <b>{item.listed ? `${item.price.toFixed(2)} SOL` : "Not minted"}</b>
                </div>
                {item.listed && (
                  <button
                    className="buy-button"
                    onClick={() =>
                      setNotice(
                        "Buy action is disabled. Connect a reviewed wallet flow before any transaction.",
                      )
                    }
                  >
                    View item <span>↗</span>
                  </button>
                )}
              </article>
            ))}
        </div>
        {shown.length === 0 && (
          <div className="empty-market">
            Nothing here yet. Try another filter or make a local preview.
          </div>
        )}
      </section>
      <footer className="mint-footer">
        <span>✳ SOLMINT / PORTFOLIO SHOWCASE</span>
        <span>Fictional listings · no on-chain activity</span>
      </footer>
      {selected && (
        <div
          className="mint-modal-backdrop"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) setSelected(null);
          }}
        >
          <section
            className="nft-detail"
            role="dialog"
            aria-modal="true"
            aria-labelledby="nft-detail-title"
          >
            <button className="mint-close" onClick={() => setSelected(null)} aria-label="Close">
              ×
            </button>
            <div className={`detail-art art-${selected.accent}`}>
              <span>{selected.art}</span>
            </div>
            <div className="detail-copy">
              <span className="mint-eyebrow">
                {selected.collection.toUpperCase()} · SAMPLE ITEM
              </span>
              <h2 id="nft-detail-title">{selected.name}</h2>
              <p>
                Created by <b>{selected.creator}</b>
              </p>
              <p>
                A visual study exploring shape, color, and the feeling of a quiet moment. This
                description is sample marketplace content.
              </p>
              <div className="detail-attributes">
                <span>
                  Series<b>{selected.collection}</b>
                </span>
                <span>
                  Category<b>{selected.category}</b>
                </span>
                <span>
                  Status<b>{selected.listed ? "Sample listing" : "Unminted preview"}</b>
                </span>
              </div>
              <div className="detail-price">
                <small>{selected.listed ? "ILLUSTRATIVE PRICE" : "LOCAL PREVIEW"}</small>
                <strong>
                  {selected.listed ? `${selected.price.toFixed(2)} SOL` : "Not on-chain"}
                </strong>
              </div>
              <button
                className="connect-button wide"
                onClick={() =>
                  setNotice(
                    "Transactions are disabled: no wallet, smart contract, or chain is connected.",
                  )
                }
              >
                {selected.listed ? "Buy action unavailable" : "Preview only"}
              </button>
            </div>
          </section>
        </div>
      )}
      {mintOpen && (
        <div
          className="mint-modal-backdrop"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) setMintOpen(false);
          }}
        >
          <form className="create-modal" onSubmit={createPreview}>
            <button
              type="button"
              className="mint-close"
              onClick={() => setMintOpen(false)}
              aria-label="Close"
            >
              ×
            </button>
            <span className="mint-eyebrow">CREATE / LOCAL PREVIEW</span>
            <h2>Start with an idea.</h2>
            <p>
              This creates a local draft preview. It will not mint an NFT, upload files, or list
              anything on Solana.
            </p>
            <label>
              Artwork name
              <input
                value={mintName}
                onChange={(event) => setMintName(event.target.value)}
                maxLength={60}
                required
                placeholder="Give your piece a name"
              />
            </label>
            <label>
              Description
              <textarea
                value={mintDescription}
                onChange={(event) => setMintDescription(event.target.value)}
                maxLength={300}
                rows={3}
                placeholder="A few words about the piece"
              />
            </label>
            <label className="upload-label">
              {fileName
                ? `Selected locally: ${fileName}`
                : "Choose an image file (local preview only)"}
              <input
                type="file"
                accept="image/*"
                onChange={(event) => setFileName(event.currentTarget.files?.[0]?.name ?? "")}
              />
            </label>
            <button type="submit" className="connect-button wide">
              Create local preview
            </button>
          </form>
        </div>
      )}
      {notice && (
        <div className="mint-toast" role="status">
          {notice}
          <button onClick={() => setNotice("")} aria-label="Dismiss">
            ×
          </button>
        </div>
      )}
    </main>
  );
}
