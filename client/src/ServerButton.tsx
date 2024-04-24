import React from 'react';

export function ServerButton() {
  const [serverOn, setServerOn] = React.useState(false);
  if (serverOn) {
    return (
      <div
        onClick={() => {
          setServerOn(false);
        }}
      >
        Turn Server Off
      </div>
    );
  } else {
    return (
      <div
        onClick={() => {
          setServerOn(true);
        }}
      >
        Turn Server On
      </div>
    );
  }
}
