import type { SavedAccount } from "./types";
import { Icon } from "../../components/Icon";

interface Props {
  accounts: SavedAccount[];
  activeUserId?: number;
  busy: boolean;
  onChoose: (id: number) => void;
  onForget: (id: number) => void;
}

export function AccountList({ accounts, activeUserId, busy, onChoose, onForget }: Props) {
  return (
    <div className="saved-accounts">
      {accounts.map((account) => (
        <div className="saved-account" key={account.id}>
          <button className="account-choice" disabled={busy || account.id === activeUserId}
            onClick={() => onChoose(account.id)}>
            <span className="avatar">{account.username.slice(0, 2).toUpperCase()}</span>
            <span><strong>{account.username}</strong><small>{account.farm_name}</small></span>
            {account.id === activeUserId && <span className="account-current">Activ</span>}
          </button>
          {account.id !== activeUserId && <button className="icon-button" disabled={busy} onClick={() => onForget(account.id)}
            title="Elimină din accesul rapid" aria-label={`Elimină contul ${account.username} din accesul rapid`}>
            <Icon name="close" size={16} />
          </button>}
        </div>
      ))}
    </div>
  );
}
