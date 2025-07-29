import React from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import {
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { ServerCard } from './ServerCard';

interface Server {
  id: string;
  name: string;
  description: string;
  hasCustomImg: boolean;
  imgUrl: string;
  version: string;
  serverType: string;
  loaderVersion: string;
  isOnline: boolean;
  playerCount: number;
  maxPlayers: number;
  memory?: number;
}

interface SortableServerItemProps {
  server: Server;
  onEdit: (serverId: string) => void;
  onOpenFolder: (serverId: string) => void;
  onDelete: (serverId: string) => void;
  onClick: (serverId: string) => void;
}

const SortableServerItem: React.FC<SortableServerItemProps> = ({
  server,
  onEdit,
  onOpenFolder,
  onDelete,
  onClick,
}) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: server.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.8 : 1,
    zIndex: isDragging ? 1000 : 'auto',
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className="touch-none"
    >
      <ServerCard
        name={server.name}
        description={server.description}
        hasCustomImg={server.hasCustomImg}
        imgUrl={server.imgUrl}
        serverType={server.serverType}
        version={server.version}
        loaderVersion={server.loaderVersion}
        playerCount={server.playerCount}
        maxPlayers={server.maxPlayers}
        onEdit={() => onEdit(server.id)}
        onOpenFolder={() => onOpenFolder(server.id)}
        onDelete={() => onDelete(server.id)}
        onClick={() => onClick(server.id)}
      />
    </div>
  );
};

interface DraggableServerListProps {
  servers: Server[];
  onEdit: (serverId: string) => void;
  onOpenFolder: (serverId: string) => void;
  onDelete: (serverId: string) => void;
  onClick: (serverId: string) => void;
  onReorder: (newOrder: Server[]) => void;
}

export const DraggableServerList: React.FC<DraggableServerListProps> = ({
  servers,
  onEdit,
  onOpenFolder,
  onDelete,
  onClick,
  onReorder,
}) => {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // Minimum drag distance to activate
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = servers.findIndex((server) => server.id === active.id);
      const newIndex = servers.findIndex((server) => server.id === over.id);

      if (oldIndex !== -1 && newIndex !== -1) {
        const newOrder = arrayMove(servers, oldIndex, newIndex);
        onReorder(newOrder);
      }
    }
  };

  return (
    <div className="space-y-4">
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={servers.map(s => s.id)} strategy={verticalListSortingStrategy}>
          {servers.map((server) => (
            <SortableServerItem
              key={server.id}
              server={server}
              onEdit={onEdit}
              onOpenFolder={onOpenFolder}
              onDelete={onDelete}
              onClick={onClick}
            />
          ))}
        </SortableContext>
      </DndContext>
    </div>
  );
};