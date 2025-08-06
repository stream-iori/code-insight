package com.example.entity;

import com.example.model.User;
import java.util.List;

/**
 * Enhanced User entity that extends BaseEntity
 * Demonstrates inheritance from abstract base class
 */
public class EnhancedUser extends BaseEntity {
    private Long id;
    private String username;
    private String email;
    private String firstName;
    private String lastName;
    private User.UserRole role;
    private List<String> tags;
    private String profileImageUrl;
    
    public EnhancedUser() {}
    
    public EnhancedUser(String username, String email, String firstName, String lastName) {
        this.username = username;
        this.email = email;
        this.firstName = firstName;
        this.lastName = lastName;
        this.role = User.UserRole.USER;
    }
    
    @Override
    public Long getId() {
        return id;
    }
    
    @Override
    public void setId(Long id) {
        this.id = id;
    }
    
    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }
    
    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
    
    public String getFirstName() { return firstName; }
    public void setFirstName(String firstName) { this.firstName = firstName; }
    
    public String getLastName() { return lastName; }
    public void setLastName(String lastName) { this.lastName = lastName; }
    
    public User.UserRole getRole() { return role; }
    public void setRole(User.UserRole role) { this.role = role; }
    
    public List<String> getTags() { return tags; }
    public void setTags(List<String> tags) { this.tags = tags; }
    
    public String getProfileImageUrl() { return profileImageUrl; }
    public void setProfileImageUrl(String profileImageUrl) { this.profileImageUrl = profileImageUrl; }
    
    public String getFullName() {
        return (firstName != null ? firstName : "") + 
               (lastName != null ? " " + lastName : "");
    }
    
    /**
     * Business logic method to check if user can perform admin actions
     */
    public boolean canPerformAdminActions() {
        return role == User.UserRole.ADMIN;
    }
    
    /**
     * Business logic method to validate user profile completeness
     */
    public boolean isProfileComplete() {
        return username != null && !username.trim().isEmpty() &&
               email != null && !email.trim().isEmpty() &&
               firstName != null && !firstName.trim().isEmpty();
    }
}