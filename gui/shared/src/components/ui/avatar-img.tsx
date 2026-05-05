import React from 'react';
import defaultAvatar from '../../../public/avatar.png';

type AvatarProps = {
  src?: string;
  alt?: string;
  sizeClass?: string; // e.g. 'h-6'
  wrapperClass?: string;
};

export default function Avatar({ src = defaultAvatar, alt = 'avatar', sizeClass = 'h-6', wrapperClass = ' bg-primary-500' }: AvatarProps) {
  return (
    <div className={`${wrapperClass} rounded-full ${sizeClass} aspect-square overflow-hidden contrast-150 saturate-50`}>
      <img src={src} alt={alt} className="w-full h-full object-cover rounded-full mix-blend-multiply " />
    </div>
  );
}
