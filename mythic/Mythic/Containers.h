#pragma once

#include <algorithm>
#include <cassert>
#include <cstddef>
#include <initializer_list>
#include <iterator>
#include <memory>
#include <new>
#include <utility>

namespace mythic {

/// Growable vector with the first N elements stored inline (SmallVector-style).
/// Preserves Unreal TArray<T, TInlineAllocator<N>> nested-embed semantics.
template <typename T, std::size_t N>
class InlineVector {
public:
    using value_type = T;
    using size_type = std::size_t;
    using reference = T&;
    using const_reference = const T&;
    using pointer = T*;
    using const_pointer = const T*;
    using iterator = T*;
    using const_iterator = const T*;

    InlineVector() = default;

    InlineVector(std::initializer_list<T> init) {
        reserve(init.size());
        for (const T& value : init) {
            push_back(value);
        }
    }

    InlineVector(const InlineVector& other) {
        reserve(other.size_);
        for (size_type i = 0; i < other.size_; ++i) {
            ::new (static_cast<void*>(data() + i)) T(other.data()[i]);
        }
        size_ = other.size_;
    }

    InlineVector(InlineVector&& other) noexcept {
        move_from(std::move(other));
    }

    InlineVector& operator=(const InlineVector& other) {
        if (this == &other) {
            return *this;
        }
        clear();
        reserve(other.size_);
        for (size_type i = 0; i < other.size_; ++i) {
            ::new (static_cast<void*>(data() + i)) T(other.data()[i]);
        }
        size_ = other.size_;
        return *this;
    }

    InlineVector& operator=(InlineVector&& other) noexcept {
        if (this == &other) {
            return *this;
        }
        clear();
        deallocate_heap();
        move_from(std::move(other));
        return *this;
    }

    ~InlineVector() {
        clear();
        deallocate_heap();
    }

    [[nodiscard]] size_type size() const {
        return size_;
    }

    [[nodiscard]] size_type capacity() const {
        return capacity_;
    }

    [[nodiscard]] bool empty() const {
        return size_ == 0;
    }

    [[nodiscard]] pointer data() {
        return heap_ != nullptr ? heap_ : inline_ptr();
    }

    [[nodiscard]] const_pointer data() const {
        return heap_ != nullptr ? heap_ : inline_ptr();
    }

    reference operator[](size_type index) {
        return data()[index];
    }

    const_reference operator[](size_type index) const {
        return data()[index];
    }

    reference front() {
        return data()[0];
    }

    const_reference front() const {
        return data()[0];
    }

    reference back() {
        return data()[size_ - 1];
    }

    const_reference back() const {
        return data()[size_ - 1];
    }

    iterator begin() {
        return data();
    }

    const_iterator begin() const {
        return data();
    }

    iterator end() {
        return data() + size_;
    }

    const_iterator end() const {
        return data() + size_;
    }

    void clear() {
        pointer ptr = data();
        for (size_type i = 0; i < size_; ++i) {
            ptr[i].~T();
        }
        size_ = 0;
    }

    void reserve(size_type new_capacity) {
        if (new_capacity <= capacity_) {
            return;
        }
        grow(new_capacity);
    }

    void push_back(const T& value) {
        emplace_back(value);
    }

    void push_back(T&& value) {
        emplace_back(std::move(value));
    }

    template <typename... Args>
    reference emplace_back(Args&&... args) {
        if (size_ == capacity_) {
            grow(capacity_ == 0 ? std::max(N, size_type{1}) : capacity_ * 2);
        }
        pointer ptr = data();
        ::new (static_cast<void*>(ptr + size_)) T(std::forward<Args>(args)...);
        ++size_;
        return ptr[size_ - 1];
    }

    void pop_back() {
        assert(size_ > 0);
        data()[size_ - 1].~T();
        --size_;
    }

    void erase(iterator position) {
        assert(position >= begin() && position < end());
        iterator next = position + 1;
        while (next != end()) {
            *position = std::move(*next);
            ++position;
            ++next;
        }
        pop_back();
    }

    [[nodiscard]] bool contains(const T& value) const {
        return std::find(begin(), end(), value) != end();
    }

    /// Insert if not already present (small-set helper).
    bool insert_unique(const T& value) {
        if (contains(value)) {
            return false;
        }
        push_back(value);
        return true;
    }

private:
    pointer inline_ptr() {
        return std::launder(reinterpret_cast<pointer>(inline_storage_));
    }

    const_pointer inline_ptr() const {
        return std::launder(reinterpret_cast<const_pointer>(inline_storage_));
    }

    void grow(size_type new_capacity) {
        assert(new_capacity > capacity_);
        pointer new_heap = static_cast<pointer>(::operator new(new_capacity * sizeof(T)));
        pointer old = data();
        for (size_type i = 0; i < size_; ++i) {
            ::new (static_cast<void*>(new_heap + i)) T(std::move(old[i]));
            old[i].~T();
        }
        deallocate_heap();
        heap_ = new_heap;
        capacity_ = new_capacity;
    }

    void deallocate_heap() {
        if (heap_ != nullptr) {
            ::operator delete(heap_);
            heap_ = nullptr;
        }
        capacity_ = N;
    }

    void move_from(InlineVector&& other) noexcept {
        size_ = other.size_;
        if (other.heap_ != nullptr) {
            heap_ = other.heap_;
            capacity_ = other.capacity_;
            other.heap_ = nullptr;
            other.capacity_ = N;
            other.size_ = 0;
        } else {
            heap_ = nullptr;
            capacity_ = N;
            pointer dest = inline_ptr();
            pointer src = other.inline_ptr();
            for (size_type i = 0; i < size_; ++i) {
                ::new (static_cast<void*>(dest + i)) T(std::move(src[i]));
                src[i].~T();
            }
            other.size_ = 0;
        }
    }

    alignas(T) unsigned char inline_storage_[sizeof(T) * (N > 0 ? N : 1)]{};
    pointer heap_ = nullptr;
    size_type size_ = 0;
    size_type capacity_ = N;
};

/// Fixed-capacity array with no heap growth (Unreal TFixedAllocator).
template <typename T, std::size_t N>
class FixedArray {
public:
    using value_type = T;
    using size_type = std::size_t;
    using reference = T&;
    using const_reference = const T&;
    using pointer = T*;
    using const_pointer = const T*;
    using iterator = T*;
    using const_iterator = const T*;

    static constexpr size_type max_size() {
        return N;
    }

    FixedArray() = default;

    FixedArray(std::initializer_list<T> init) {
        assert(init.size() <= N);
        for (const T& value : init) {
            push_back(value);
        }
    }

    FixedArray(const FixedArray& other) {
        for (size_type i = 0; i < other.size_; ++i) {
            ::new (static_cast<void*>(data() + i)) T(other.data()[i]);
        }
        size_ = other.size_;
    }

    FixedArray& operator=(const FixedArray& other) {
        if (this == &other) {
            return *this;
        }
        clear();
        for (size_type i = 0; i < other.size_; ++i) {
            ::new (static_cast<void*>(data() + i)) T(other.data()[i]);
        }
        size_ = other.size_;
        return *this;
    }

    ~FixedArray() {
        clear();
    }

    [[nodiscard]] size_type size() const {
        return size_;
    }

    [[nodiscard]] bool empty() const {
        return size_ == 0;
    }

    [[nodiscard]] pointer data() {
        return std::launder(reinterpret_cast<pointer>(storage_));
    }

    [[nodiscard]] const_pointer data() const {
        return std::launder(reinterpret_cast<const_pointer>(storage_));
    }

    reference operator[](size_type index) {
        return data()[index];
    }

    const_reference operator[](size_type index) const {
        return data()[index];
    }

    iterator begin() {
        return data();
    }

    const_iterator begin() const {
        return data();
    }

    iterator end() {
        return data() + size_;
    }

    const_iterator end() const {
        return data() + size_;
    }

    void clear() {
        pointer ptr = data();
        for (size_type i = 0; i < size_; ++i) {
            ptr[i].~T();
        }
        size_ = 0;
    }

    void push_back(const T& value) {
        assert(size_ < N);
        ::new (static_cast<void*>(data() + size_)) T(value);
        ++size_;
    }

    template <typename... Args>
    reference emplace_back(Args&&... args) {
        assert(size_ < N);
        ::new (static_cast<void*>(data() + size_)) T(std::forward<Args>(args)...);
        ++size_;
        return data()[size_ - 1];
    }

    [[nodiscard]] bool contains(const T& value) const {
        return std::find(begin(), end(), value) != end();
    }

    bool insert_unique(const T& value) {
        if (contains(value)) {
            return false;
        }
        push_back(value);
        return true;
    }

private:
    alignas(T) unsigned char storage_[sizeof(T) * (N > 0 ? N : 1)]{};
    size_type size_ = 0;
};

} // namespace mythic
