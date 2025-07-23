package com.example.repository;

import com.example.model.User;
import java.util.List;

/**
 * Interface for database operations with Users
 * This shows what an interface looks like in Java
 */
public interface UserRepository {
    // Method signatures - what the implementation must provide
    User findById(Long id);
    List<User> findAll();
    User save(User user);
    void deleteById(Long id);
}