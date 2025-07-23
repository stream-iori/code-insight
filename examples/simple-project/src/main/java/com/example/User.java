package com.example.model;

/**
 * A simple User entity - represents a user in our system
 * This is what a basic Java class looks like
 */
public class User {
    // Fields (variables) of this class
    private Long id;
    private String username;
    private String email;
    
    // Constructor - how to create a new User
    public User(Long id, String username, String email) {
        this.id = id;
        this.username = username;
        this.email = email;
    }
    
    // Getters and setters - ways to access the fields
    public Long getId() {
        return id;
    }
    
    public void setId(Long id) {
        this.id = id;
    }
    
    public String getUsername() {
        return username;
    }
    
    public void setUsername(String username) {
        this.username = username;
    }
    
    public String getEmail() {
        return email;
    }
    
    public void setEmail(String email) {
        this.email = email;
    }
}