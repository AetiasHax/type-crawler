namespace std {
    template <typename T>
    class vector {
        T* elements;
        int size;
        int capacity;
    };
}

struct Entity;

class World {
    std::vector<Entity*> activeEntities;
    std::vector<Entity*> frozenEntities;
};
