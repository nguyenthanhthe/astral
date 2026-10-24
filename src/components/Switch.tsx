interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  title?: string;
  id?: string;
}

/** Toggle switch per the design contract (32×18 track, dark ink on accent). */
export function Switch({ checked, onChange, label, title, id }: SwitchProps) {
  return (
    <label className="switch" title={title}>
      <input
        id={id}
        type="checkbox"
        className="switch__input"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className="switch__track" aria-hidden="true" />
      <span className="switch__label">{label}</span>
    </label>
  );
}
